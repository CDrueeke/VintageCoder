// SPDX-License-Identifier: GPL-3.0-or-later
//! Disk-Studio ohne Oberfläche: D64, D71, D81 und ATR öffnen, Verzeichnis und Belegung lesen, prüfen, Dateien ablegen, Image speichern
//! und Dateien ins Projekt übernehmen. Verzeichnis und Belegung sind für alle vier Formate derselbe Kern; die Besonderheiten (Spur/Sektor
//! bzw. Sektornummer, PETSCII bzw. ATASCII, Kettenzeiger) kommen aus `vc_disk::AnyDisk`. Die Oberfläche zeigt nur an.
use std::path::{Path, PathBuf};
use vc_disk::{Addr, AnyDisk, Analysis, Cell, FileInfo, Format};
use vc_ide::diskimage::{detect_basic, import_data, BasicInfo, Image, ImportOpts};

/// Ein geöffnetes Diskettenabbild mit allem, was das Studio anzeigt
pub struct DiskDoc {
    /// Kette der gewählten Datei auf der Belegungskarte zeigen (nur Anzeige, nicht in der Datei)
    pub kette_an: bool,
    /// Ort der Imagedatei; `None` = neu, noch nicht gespeichert
    pub path: Option<PathBuf>,
    pub disk: Result<AnyDisk, String>,
    /// Lesesicht für Dateiinhalte und Übernahme ins Projekt
    pub img: Option<Image>,
    pub files: Vec<FileInfo>,
    pub an: Analysis,
    pub dirty: bool,
    /// gewählte Datei im Verzeichnis
    pub sel_file: Option<usize>,
    /// Sektor im Inspektor und besuchte Sektoren („Zurück“)
    pub cur: Addr,
    pub history: Vec<Addr>,
    /// zur Übernahme ins Projekt vorgemerkt
    pub checked: Vec<bool>,
    pub basic: Vec<Option<BasicInfo>>,
    pub opts: ImportOpts,
    data: Option<PathBuf>,
    /// Ergebnis von „Validieren“ (leer = in Ordnung)
    pub report: Option<Vec<String>>,
    /// letzte Meldung (Erfolg, Text)
    pub note: Option<(bool, String)>,
    /// Rückgängig/Wiederholen: Abbilder (Bytes) vor bzw. nach einer Änderung; `gespeichert` = Tiefe des Rückgängig-Verlaufs beim letzten Speichern
    pub undo: Vec<Vec<u8>>,
    pub redo: Vec<Vec<u8>>,
    gespeichert: Option<usize>,
}

// ---------------------------------------------------------------------------------------------------------------------
// Darstellung von Namen und Zahlen

/// PETSCII (Großschrift/Grafik-Zeichensatz) in lesbaren Text; Grafikzeichen erscheinen als »▒«
pub fn petscii_text(b: &[u8]) -> String {
    b.iter().map(|&c| match c {
        0x20..=0x40 | 0x41..=0x5A | 0x5B | 0x5D => c as char,
        0x5C => '£', 0x5E => '↑', 0x5F => '←',
        0xA0 => ' ',
        _ => '▒',
    }).collect()
}

pub fn atascii_text(b: &[u8]) -> String { b.iter().map(|&c| if (0x20..0x7F).contains(&c) { c as char } else { '·' }).collect() }

pub fn name_text(atari: bool, b: &[u8]) -> String { if atari { atascii_text(b) } else { petscii_text(b) } }

/// „3,5 KB“ wie im Mockup
pub fn size_text(n: usize) -> String { if n < 1024 { vc_i18n::t!("diskdoc-n-b", n = &n) } else { vc_i18n::t!("diskdoc-p1-kb", p1 = vc_i18n::num((n as f64 / 1024.0) as f64, 1)).replace('.', ",") } }

pub fn addr_text(f: Format, a: Addr) -> String { if f.is_atari() { format!("{}", AnyDisk::atr_number(a)) } else { format!("{}/{:02}", a.track, a.sector) } }

pub fn geometry_text(d: &AnyDisk) -> String {
    match d.format() {
        Format::D64 => vc_i18n::t!("diskdoc-1541-35-spuren-683-sektoren").into(),
        Format::D71 => vc_i18n::t!("diskdoc-1571-70-spuren-2-seiten-1366").into(),
        Format::D81 => vc_i18n::t!("diskdoc-1581-80-spuren-zu-40-sektoren").into(),
        Format::Atr => match d { AnyDisk::Atr(a) => {
            let dos = a.kind().name();
            let mut t: String = if dos.is_empty() { vc_i18n::t!("diskdoc-atari-ohne-dos", n = &a.sector_count(), ssz = &a.sector_size()).into() } else { vc_i18n::t!("diskdoc-atari-geometrie", dos = dos, n = &a.sector_count(), ssz = &a.sector_size()).into() };
            if a.boot_128() { t.push_str(&vc_i18n::t!("diskdoc-atari-boot-128")); }
            t
        } _ => String::new() },
    }
}

/// Zielsystem für Farbe und Emulator
pub fn target_for(f: Format) -> &'static str { if f.is_atari() { "atari8" } else { "c64" } }

/// Format aus der Kennung des Formulars „Neue Diskette“ (`d64`, `d71`, `d81`, `atr`)
pub fn format_aus_kennung(k: &str) -> Format { match k { "d71" => Format::D71, "d81" => Format::D81, "atr" => Format::Atr, _ => Format::D64 } }

pub fn ist_programm(img: &Image, kind: &str) -> bool { kind == "PRG" || (img.atari && kind == "DOS") }

/// Wer ein Sektor ist, in Worten (Inspektor, Hinweis über der Belegungskarte)
pub fn zelle_text(atari: bool, c: Cell, files: &[FileInfo]) -> String {
    match c {
        Cell::Free => vc_i18n::t!("diskdoc-frei"),
        Cell::System => if atari { vc_i18n::t!("diskdoc-boot-vtoc").into() } else { vc_i18n::t!("diskdoc-kopf-bam").into() },
        Cell::Dir => vc_i18n::t!("diskdoc-verzeichnis").into(),
        Cell::File(i) => files.get(i).map(|f| format!("{} ({})", name_text(atari, &f.name), f.kind)).unwrap_or_default(),
        Cell::Orphan => vc_i18n::t!("diskdoc-belegt-aber-ohne-datei").into(),
        Cell::Conflict => vc_i18n::t!("diskdoc-von-mehreren-ketten-erreicht").into(),
    }
}

/// Eine Zeile des Sektor-Inspektors: Versatz, Bytes (mit Kennzeichnung der Zeigerbytes), Text
pub struct HexZeile { pub versatz: usize, pub bytes: Vec<(u8, bool)>, pub text: String }

impl DiskDoc {
    fn leer(data: Option<PathBuf>) -> DiskDoc {
        DiskDoc { kette_an: true, path: None, disk: Err(String::new()), img: None, files: vec![], an: Analysis { cells: vec![], free: 0, used: 0, problems: vec![], interleave: None },
            dirty: false, sel_file: None, cur: Addr { track: 1, sector: 0 }, history: vec![], checked: vec![], basic: vec![], opts: ImportOpts::default(), data, report: None, note: None, undo: vec![], redo: vec![], gespeichert: Some(0) }
    }

    /// Image aus einer Datei öffnen; ein unlesbares Image bleibt als Fehlertext im Dokument (`disk` = `Err`)
    pub fn oeffnen(path: &Path, data: Option<PathBuf>) -> DiskDoc {
        let mut s = Self::leer(data);
        let r = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display())).and_then(|b| AnyDisk::from_file_bytes(&path.to_string_lossy(), &b));
        s.path = Some(path.to_path_buf()); s.set_disk(r); s
    }

    /// Neue, leere Diskette (noch nicht gespeichert)
    pub fn neu(format: Format, name: &str, id: &str, data: Option<PathBuf>) -> Result<DiskDoc, String> {
        let d = AnyDisk::blank(format, name, id)?;
        let mut s = Self::leer(data); s.dirty = true; s.gespeichert = None; s.set_disk(Ok(d)); Ok(s)
    }

    fn set_disk(&mut self, d: Result<AnyDisk, String>) {
        self.disk = d; self.report = None; self.sel_file = None; self.history.clear(); self.refresh();
        if let Ok(d) = &self.disk { self.cur = Self::erster_sektor(d); }
    }

    /// Erster interessanter Sektor: Kopf bzw. BAM/VTOC
    fn erster_sektor(d: &AnyDisk) -> Addr {
        match d.format() { Format::D64 | Format::D71 => Addr { track: 18, sector: 0 }, Format::D81 => Addr { track: 40, sector: 0 }, Format::Atr => AnyDisk::atr_addr(360) }
    }

    /// Nach jeder Änderung: Belegung, Verzeichnis und Lesesicht neu berechnen
    pub fn refresh(&mut self) {
        let Ok(d) = &self.disk else { self.img = None; self.files.clear(); self.checked.clear(); self.basic.clear(); return };
        self.files = d.files(); self.an = d.analyze();
        let name = self.path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| format!("neu.{}", d.format().label().to_lowercase()));
        self.img = Image::from_bytes(&name, &d.to_bytes()).ok();
        let atari = d.format().is_atari();
        self.basic = match &self.img {
            Some(i) => i.entries.iter().map(|e| match (&e.data, &self.data) { (Ok(b), Some(dd)) if e.kind == "PRG" || atari => detect_basic(dd, b, atari), _ => None }).collect(),
            None => vec![],
        };
        self.checked.resize(self.files.len(), false);
    }

    pub fn format(&self) -> Option<Format> { self.disk.as_ref().ok().map(|d| d.format()) }
    pub fn atari(&self) -> bool { self.format().is_some_and(|f| f.is_atari()) }
    /// Dateiname (oder „neu, nicht gespeichert“)
    pub fn titel(&self) -> String { self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| vc_i18n::t!("diskdoc-neu-nicht-gespeichert").into()) }
    pub fn ist_d81(&self) -> bool { self.path.as_ref().and_then(|p| p.extension()).is_some_and(|e| e.eq_ignore_ascii_case("d81")) }

    // -- Auswahl und Vormerken ---------------------------------------------------------------------------------------

    pub fn alle_vormerken(&mut self) { self.checked.iter_mut().for_each(|s| *s = true) }
    pub fn keine_vormerken(&mut self) { self.checked.iter_mut().for_each(|s| *s = false) }
    pub fn nur_programme(&mut self) {
        let Some(img) = &self.img else { return };
        for (i, e) in img.entries.iter().enumerate() { if let Some(c) = self.checked.get_mut(i) { *c = ist_programm(img, &e.kind) && e.data.is_ok() } }
    }
    pub fn nur_basic(&mut self) { for i in 0..self.checked.len() { self.checked[i] = self.basic.get(i).is_some_and(|b| b.is_some()) } }
    pub fn vormerken(&mut self, i: usize, an: bool) {
        let ok = self.img.as_ref().and_then(|im| im.entries.get(i)).is_some_and(|e| e.data.is_ok());
        if let Some(c) = self.checked.get_mut(i) { *c = an && ok }
    }
    pub fn anzahl_vorgemerkt(&self) -> usize { self.checked.iter().filter(|s| **s).count() }
    /// Zielordner der Übernahme muss innerhalb des Projekts liegen
    pub fn zielordner_ungueltig(&self) -> bool { let d = &self.opts.dir; !d.is_empty() && (d.starts_with('/') || d.contains("..") || d.contains('\\')) }

    /// Datei im Verzeichnis wählen: ihre Kette beginnt im Inspektor
    pub fn datei_waehlen(&mut self, i: usize) { if let Some(f) = self.files.get(i) { self.sel_file = Some(i); self.history.clear(); self.cur = f.start } }

    /// Sektor wählen (Klick in der Belegungskarte): gehört er zu einer Datei, wird sie mitgewählt
    pub fn sektor_waehlen(&mut self, a: Addr) {
        if let Some(Cell::File(i)) = self.an.cell(a) { self.sel_file = Some(i) }
        self.gehe_zu(a);
    }

    pub fn gehe_zu(&mut self, a: Addr) {
        if a != self.cur { self.history.push(self.cur); if self.history.len() > 200 { self.history.remove(0); } }
        self.cur = a;
    }
    pub fn zurueck(&mut self) { if let Some(a) = self.history.pop() { self.cur = a } }
    /// Folgesektor der Kette des aktuellen Sektors
    pub fn folge(&self) -> Option<Addr> { self.disk.as_ref().ok().and_then(|d| d.next(self.cur, &self.an)) }
    pub fn folgen(&mut self) { if let Some(n) = self.folge() { self.gehe_zu(n) } }
    /// Kette der gewählten Datei in Lesereihenfolge (Sektoradressen); bricht bei einem Kreis oder fehlerhaftem Zeiger ab
    pub fn kette(&self) -> Vec<Addr> {
        let (Some(f), Ok(d)) = (self.sel_file.and_then(|i| self.files.get(i)), self.disk.as_ref()) else { return vec![] };
        let (mut v, mut a, mut gesehen) = (vec![], Some(f.start), std::collections::HashSet::new());
        while let Some(x) = a { if v.len() > 4000 || !gesehen.insert(x) { break } v.push(x); a = d.next(x, &self.an) }
        v
    }
    /// „Block n von m der Datei“ für den gewählten Sektor, wenn er in der Kette der gewählten Datei liegt
    pub fn kettenposition(&self) -> Option<(usize, usize)> { let k = self.kette(); k.iter().position(|a| *a == self.cur).map(|i| (i + 1, k.len())) }

    // -- Inspektor ----------------------------------------------------------------------------------------------------

    pub fn sektor_titel(&self) -> String { if self.atari() { vc_i18n::t!("diskdoc-sektor-p1", p1 = AnyDisk::atr_number(self.cur)) } else { vc_i18n::t!("diskdoc-sektor-p1-p2", p1 = &self.cur.track, p2 = &self.cur.sector) } }
    /// Zugehörigkeit des aktuellen Sektors in Worten
    pub fn besitzer(&self) -> Option<String> {
        let atari = self.atari();
        Some(match self.an.cell(self.cur)? {
            Cell::File(i) => self.files.get(i).map(|f| vc_i18n::t!("diskdoc-gehoert-zu-p1", p1 = name_text(atari, &f.name)))?,
            Cell::Dir => vc_i18n::t!("diskdoc-verzeichnis").into(), Cell::System => if atari { vc_i18n::t!("diskdoc-boot-bzw-vtoc").into() } else { vc_i18n::t!("diskdoc-kopf-bam").into() },
            Cell::Free => vc_i18n::t!("diskdoc-frei-laut-bam").into(), Cell::Orphan => vc_i18n::t!("diskdoc-belegt-ohne-datei").into(), Cell::Conflict => vc_i18n::t!("diskdoc-konflikt-mehrfach-belegt").into(),
        })
    }
    /// Hinweis zum Kettenzeiger des aktuellen Sektors
    pub fn zeiger_hinweis(&self) -> Option<String> {
        let data = self.disk.as_ref().ok()?.sector(self.cur)?;
        Some(if self.atari() {
            let l = data.len(); let n = ((data[l - 3] & 3) as usize) << 8 | data[l - 2] as usize;
            vc_i18n::t!("diskdoc-datei-nr-p1-folgesektor-p2-p3", p1 = data[l - 3] >> 2, p2 = if n == 0 { "–".into() } else { n.to_string() }, p3 = data[l - 1])
        } else if data[0] == 0 { vc_i18n::t!("diskdoc-kettenende-letztes-byte-p1", p1 = data[1]) } else { vc_i18n::t!("diskdoc-zeiger-p1-p2", p1 = data[0], p2 = data[1]) })
    }
    /// Hexdarstellung des aktuellen Sektors (`breite` Bytes je Zeile); `None` = außerhalb der Diskette
    pub fn sektor_zeilen(&self, breite: usize) -> Option<Vec<HexZeile>> {
        let data = self.disk.as_ref().ok()?.sector(self.cur)?;
        let zeiger: std::ops::Range<usize> = if self.atari() { data.len() - 3..data.len() } else { 0..2 };
        let breite = breite.max(1);
        Some(data.chunks(breite).enumerate().map(|(r, c)| HexZeile { versatz: r * breite, bytes: c.iter().enumerate().map(|(i, b)| (*b, zeiger.contains(&(r * breite + i)))).collect(), text: c.iter().map(|&b| if (0x20..0x7F).contains(&b) { b as char } else { '·' }).collect() }).collect())
    }
    /// Spur der Belegungskarte, die zuerst Inhalt hat (zum Scrollen)
    pub fn erste_belegte_spur(&self) -> Option<usize> { (1..=self.an.cells.len()).find(|&tr| self.an.cells[tr - 1].iter().any(|c| *c != Cell::Free)) }

    // -- Aktionen -----------------------------------------------------------------------------------------------------

    // -- Rückgängig/Wiederholen und Umordnen -------------------------------------------------------------------------------
    const VERLAUF_MAX: usize = 30;
    fn verlauf_merken(&mut self, vorher: Vec<u8>) {
        if self.gespeichert.is_some_and(|g| g > self.undo.len()) { self.gespeichert = None; }   // der gespeicherte Stand ist durch die neue Änderung nicht mehr erreichbar
        self.undo.push(vorher); self.redo.clear(); if self.undo.len() > Self::VERLAUF_MAX { self.undo.remove(0); self.gespeichert = self.gespeichert.and_then(|g| g.checked_sub(1)); }
    }
    fn abbild_setzen(&mut self, bytes: &[u8]) -> bool {
        let Some(f) = self.format() else { return false };
        match AnyDisk::from_file_bytes(&format!("x.{}", f.label().to_lowercase()), bytes) {
            Ok(d) => { let sel = self.sel_file; let marken = self.checked.clone(); self.disk = Ok(d); self.refresh(); self.sel_file = sel.filter(|&i| i < self.files.len()); if marken.len() == self.checked.len() { self.checked = marken; } true }
            Err(_) => false,
        }
    }
    fn dirty_nachfuehren(&mut self) { self.dirty = self.gespeichert != Some(self.undo.len()); }
    pub fn rueckgaengig(&mut self) -> bool {
        let Some(b) = self.undo.pop() else { return false }; let Ok(d) = &self.disk else { return false };
        let jetzt = d.to_bytes(); if !self.abbild_setzen(&b) { self.undo.push(b); return false }
        self.redo.push(jetzt); self.dirty_nachfuehren(); true
    }
    pub fn wiederholen(&mut self) -> bool {
        let Some(b) = self.redo.pop() else { return false }; let Ok(d) = &self.disk else { return false };
        let jetzt = d.to_bytes(); if !self.abbild_setzen(&b) { self.redo.push(b); return false }
        self.undo.push(jetzt); self.dirty_nachfuehren(); true
    }
    /// Verzeichniseintrag `von` an die Stelle `nach` setzen (ein Rückgängig-Schritt); gewählte Datei und Vormerkung wandern mit
    pub fn datei_verschieben(&mut self, von: usize, nach: usize) -> Result<(), String> {
        let n = self.files.len(); if von >= n || nach >= n { return Err(vc_i18n::t!("disk-eintrag-nummer-ungueltig").into()); }
        if von == nach { return Ok(()) }
        let Ok(d) = &mut self.disk else { return Err(vc_i18n::t!("disk-eintrag-nummer-ungueltig").into()) };
        let vorher = d.to_bytes(); d.move_entry(von, nach)?;
        self.verlauf_merken(vorher); self.dirty = true;
        let (sel, mut marken) = (self.sel_file, self.checked.clone());
        self.refresh();
        let m = marken.remove(von); marken.insert(nach, m); if marken.len() == self.checked.len() { self.checked = marken; }
        self.sel_file = sel.map(|s| if s == von { nach } else if von < nach && s > von && s <= nach { s - 1 } else if nach < von && s >= nach && s < von { s + 1 } else { s });
        Ok(())
    }

    pub fn validieren(&mut self) { if let Ok(d) = &self.disk { self.report = Some(d.check()) } }

    pub fn speichern_unter(&mut self, p: PathBuf) {
        let Ok(d) = &self.disk else { return };
        match vc_core::atomar::schreiben(&p, d.to_bytes()) {
            Ok(()) => { self.note = Some((true, vc_i18n::t!("diskdoc-gespeichert-p1", p1 = p.display()))); self.path = Some(p); self.dirty = false; self.gespeichert = Some(self.undo.len()); self.refresh(); }
            Err(e) => self.note = Some((false, format!("{}: {e}", p.display()))),
        }
    }
    pub fn speichern(&mut self) { if let Some(p) = self.path.clone() { self.speichern_unter(p) } }

    /// Dateien auf die Diskette schreiben (Ablegen/„Hinzufügen“); der Name wird auf die Regeln des Formats gebracht
    pub fn dateien_ablegen(&mut self, paths: &[PathBuf]) {
        let vorher = self.disk.as_ref().ok().map(|d| d.to_bytes());
        let Ok(d) = &mut self.disk else { return };
        if !d.has_filesystem() { self.note = Some((false, vc_i18n::t!("diskdoc-boot-diskette-ohne-dateisystem-dateien-lassen").into())); return }
        let (mut meldungen, mut ok) = (vec![], 0);
        for p in paths {
            let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let name = if d.format().is_atari() { p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default() } else { stem.chars().take(16).collect() };
            match std::fs::read(p).map_err(|e| e.to_string()).and_then(|b| d.add_file(&name, &b)) {
                Ok(n) => { ok += 1; meldungen.push(vc_i18n::t!("diskdoc-n-hinzugefuegt", n = &n)) }
                Err(e) => meldungen.push(format!("{}: {e}", p.display())),
            }
        }
        self.note = Some((ok > 0 && meldungen.len() == ok, meldungen.join(" · ")));
        if ok > 0 { if let Some(b) = vorher { self.verlauf_merken(b); } self.dirty = true; self.refresh() }
    }

    /// Vorgemerkte Dateien ins Projekt übernehmen; Meldungen (Erfolg, Text)
    pub fn uebernehmen(&self, root: &Path) -> Vec<(bool, String)> {
        let mut msgs = vec![];
        let Some(img) = &self.img else { return msgs };
        for (_, e) in img.entries.iter().enumerate().filter(|(i, _)| self.checked.get(*i).copied().unwrap_or(false)) {
            let r = match &e.data {
                Ok(d) => import_data(root, &img.suggested_name(e), d, ist_programm(img, &e.kind), img.atari, self.data.as_deref(), &self.opts),
                Err(m) => Err(m.clone()),
            };
            match r {
                Ok(o) => { msgs.push((true, format!("{} → {}", e.name, o.files.join(", ")))); for n in o.notes { msgs.push((true, format!("  {n}"))) } }
                Err(m) => msgs.push((false, format!("{}: {m}", e.name))),
            }
        }
        msgs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let tmp = std::env::temp_dir().join(format!("vc_diskdoc_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap(); tmp
    }

    #[test]
    fn kette_der_gewaehlten_datei_in_lesereihenfolge_mit_position() {
        let mut d = vc_disk::D64::blank("TEST", "01").unwrap(); d.add_prg("GROSS", &vec![0xEAu8; 1200]).unwrap(); d.add_prg("KLEIN", &[0x00, 0xC0, 0x60]).unwrap();
        let tmp = tmpdir("kette"); std::fs::write(tmp.join("k.d64"), d.to_bytes()).unwrap();
        let mut dok = DiskDoc::oeffnen(&tmp.join("k.d64"), None); assert!(dok.kette().is_empty(), "ohne gewählte Datei keine Kette");
        dok.datei_waehlen(0); let k = dok.kette(); assert!(k.len() >= 5 && k[0] == dok.files[0].start, "1200 Bytes brauchen mehrere Blöcke: {k:?}"); assert_eq!(dok.kettenposition(), Some((1, k.len())));
        dok.folgen(); assert_eq!(dok.kettenposition(), Some((2, k.len())), "Folgen rückt in der Kette vor");
        dok.sektor_waehlen(Addr { track: 18, sector: 0 }); assert_eq!(dok.kettenposition(), None, "Sektor außerhalb der Kette");
        dok.datei_waehlen(1); assert_eq!(dok.kette().len(), 1); let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn oeffnet_prueft_und_uebernimmt_ins_projekt() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let tmp = tmpdir("c64");
        // D64 mit BASIC-Programm (10 SYS2061 + Maschinencode) und reinem Maschinenprogramm
        let stub = [0x01u8, 0x08, 0x0B, 0x08, 0x0A, 0x00, 0x9E, b'2', b'0', b'6', b'1', 0x00, 0x00, 0x00, 0xA9, 0x00, 0x60];
        let mut d = vc_disk::D64::blank("TEST", "01").unwrap();
        d.add_prg("STUB", &stub).unwrap(); d.add_prg("MC", &[0x00, 0xC0, 0xEE, 0x20, 0xD0, 0x60]).unwrap();
        std::fs::write(tmp.join("t.d64"), d.to_bytes()).unwrap();
        let mut dok = DiskDoc::oeffnen(&tmp.join("t.d64"), Some(data));
        assert_eq!(dok.basic.iter().map(|b| b.is_some()).collect::<Vec<_>>(), vec![true, false]);
        assert_eq!(dok.files.iter().map(|f| f.autostart).collect::<Vec<_>>(), vec![true, false]);
        dok.validieren(); assert_eq!(dok.report, Some(vec![]), "frisch angelegtes Image ist in Ordnung");
        dok.alle_vormerken(); assert_eq!(dok.anzahl_vorgemerkt(), 2);
        let msgs = dok.uebernehmen(&tmp);
        assert!(msgs.iter().all(|m| m.0), "{msgs:?}");
        for f in ["import/stub.bas", "import/stub.prg", "import/mc.prg"] { assert!(tmp.join(f).exists(), "{f} fehlt: {msgs:?}"); }
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn boot_diskette_hat_kein_dateisystem() {
        let tmp = tmpdir("boot");
        let mut a = vc_disk::Atr::blank(); a.sector_mut(1)[0] = 0x55; // Boot-Diskette ohne DOS-2-VTOC
        std::fs::write(tmp.join("b.atr"), a.to_bytes()).unwrap();
        let mut s = DiskDoc::oeffnen(&tmp.join("b.atr"), None);
        assert!(s.disk.as_ref().is_ok_and(|d| !d.has_filesystem()));
        assert!(s.files.is_empty() && s.an.problems.is_empty());
        std::fs::write(tmp.join("x.bin"), [1u8, 2, 3]).unwrap(); s.dateien_ablegen(&[tmp.join("x.bin")]);
        assert!(!s.dirty && s.note.as_ref().is_some_and(|n| !n.0 && n.1.contains("Boot-Diskette")), "{:?}", s.note);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn atari_ablegen_speichern_und_neu_mit_kette_folgen() {
        let tmp = tmpdir("atr");
        let mut a = vc_disk::Atr::new_dos2();
        a.dos2_add("AUTORUN.SYS", &[0xFF, 0xFF, 0, 0x20, 0x02, 0x20, 0xA9, 0x00, 0x60]).unwrap();
        std::fs::write(tmp.join("t.atr"), a.to_bytes()).unwrap();
        let mut s = DiskDoc::oeffnen(&tmp.join("t.atr"), None);
        assert_eq!(s.files.len(), 1);
        // Datei ablegen: Name wird auf DOS-2-Regeln gebracht, Image ist danach gültig
        std::fs::write(tmp.join("level one.dat"), vec![7u8; 400]).unwrap();
        s.dateien_ablegen(&[tmp.join("level one.dat")]);
        assert!(s.dirty && s.files.len() == 2 && s.an.problems.is_empty(), "{:?} {:?}", s.note, s.an.problems);
        s.speichern(); assert!(!s.dirty);
        assert_eq!(vc_disk::Atr::from_bytes(&std::fs::read(tmp.join("t.atr")).unwrap()).unwrap().dos2_entries().len(), 2);
        // neue D81 anlegen, Datei ablegen, Kette folgen
        let mut n = DiskDoc::neu(Format::D81, "NEU", "01", None).unwrap();
        std::fs::write(tmp.join("prog.prg"), vec![1u8; 1000]).unwrap();
        n.dateien_ablegen(&[tmp.join("prog.prg")]);
        assert_eq!(n.files.len(), 1); n.datei_waehlen(0);
        let mut hops = 0; while n.folge().is_some() { n.folgen(); hops += 1 }
        assert_eq!(hops, 3); n.zurueck(); assert_eq!(n.history.len(), 2);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn namen_zahlen_und_sektoransicht() {
        assert_eq!(petscii_text(b"HALLO 1"), "HALLO 1");
        assert_eq!(petscii_text(&[0x41, 0xC1, 0xA0]), "A▒ ");
        assert_eq!(atascii_text(&[b'A', 0x9B]), "A·");
        assert_eq!(size_text(3584), "3,5 KB");
        let d = DiskDoc::neu(Format::D64, "NEU", "01", None).unwrap();
        let z = d.sektor_zeilen(16).unwrap(); assert_eq!(z.len(), 16); assert_eq!(d.sektor_zeilen(8).unwrap().len(), 32); assert!(z[0].bytes[0].1 && z[0].bytes[1].1 && !z[0].bytes[2].1, "Zeigerbytes gekennzeichnet");
        assert_eq!(d.sektor_titel(), "Sektor 18/0"); assert_eq!(d.besitzer().as_deref(), Some("Kopf / BAM")); assert_eq!(d.erste_belegte_spur(), Some(18));
        assert!(d.zeiger_hinweis().is_some_and(|h| h.starts_with("Zeiger 18/1")), "{:?}", d.zeiger_hinweis());
        let mut m = DiskDoc::neu(Format::D64, "N", "01", None).unwrap(); m.opts.dir = "../aus".into(); assert!(m.zielordner_ungueltig()); m.opts.dir = "import".into(); assert!(!m.zielordner_ungueltig());
        assert_eq!(format_aus_kennung("atr"), Format::Atr); assert_eq!(target_for(Format::Atr), "atari8");
    }
}
