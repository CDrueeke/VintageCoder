// SPDX-License-Identifier: GPL-3.0-or-later
//! Diskettenimages (D64, D71, D81, ATR mit DOS 2) öffnen, Verzeichnis lesen und Dateien ins Projekt übernehmen.
use std::path::Path;
use vc_disk::{Atr, Disk, Kind, D64};

/// Eine Datei auf der Diskette; die Daten sind beim Öffnen schon gelesen (Fehler stehen je Datei).
pub struct ImageEntry {
    pub name: String,
    /// PRG, SEQ, USR, REL, DEL (Commodore) bzw. "DOS" (Atari)
    pub kind: String,
    /// Größe laut Verzeichnis: Blöcke (Commodore) oder Sektoren (Atari)
    pub blocks: u16,
    pub data: Result<Vec<u8>, String>,
    pub locked: bool,
}

impl ImageEntry {
    /// Ladeadresse: PRG (zwei Bytes vorweg) bzw. erstes Segment einer Atari-Binärdatei ($FFFF, Start, Ende)
    pub fn load_address(&self) -> Option<u16> {
        let d = self.data.as_ref().ok()?;
        if self.kind == "DOS" { if d.len() >= 6 && d[0] == 0xFF && d[1] == 0xFF { Some(d[2] as u16 | (d[3] as u16) << 8) } else { None } }
        else if self.kind == "PRG" && d.len() >= 2 { Some(d[0] as u16 | (d[1] as u16) << 8) } else { None }
    }
    pub fn size(&self) -> usize { self.data.as_ref().map(|d| d.len()).unwrap_or(0) }
}

pub struct Image {
    /// z. B. "D64", "ATR (DOS 2)"
    pub format: &'static str,
    /// Disketten-Name (nur Commodore)
    pub title: String,
    pub free: String,
    pub atari: bool,
    pub entries: Vec<ImageEntry>,
}

/// Erkennt Image-Dateien an der Endung.
pub fn is_image_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.ends_with(".d64") || n.ends_with(".d71") || n.ends_with(".d81") || n.ends_with(".atr")
}

impl Image {
    pub fn open(path: &Path) -> Result<Image, String> {
        let b = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Image::from_bytes(&path.to_string_lossy(), &b)
    }

    pub fn from_bytes(name: &str, b: &[u8]) -> Result<Image, String> {
        let n = name.to_ascii_lowercase();
        if n.ends_with(".atr") {
            let a = Atr::from_bytes(b)?;
            let entries = a.dos2_entries().into_iter().map(|e| ImageEntry {
                data: a.dos2_read(&e.name), name: e.name, kind: "DOS".into(), blocks: e.sectors, locked: e.locked }).collect();
            Ok(Image { format: "ATR (DOS 2)", title: String::new(), free: format!("{} Sektoren frei", a.free_sectors()), atari: true, entries })
        } else if n.ends_with(".d64") {
            let d = D64::from_bytes(b)?;
            let entries = d.entries().into_iter().map(|e| ImageEntry { data: d.read(&e.name), name: e.name, kind: e.kind.into(), blocks: e.blocks, locked: false }).collect();
            Ok(Image { format: "D64", title: d.disk_name(), free: format!("{} Blöcke frei", d.free_blocks()), atari: false, entries })
        } else if let Some(kind) = Kind::from_extension(&n) {
            let d = Disk::from_bytes(kind, b)?;
            let entries = d.entries().into_iter().map(|e| ImageEntry { data: d.read(&e.name), name: e.name, kind: e.kind.into(), blocks: e.blocks, locked: false }).collect();
            Ok(Image { format: if kind == Kind::D71 { "D71" } else { "D81" }, title: d.disk_name(), free: format!("{} Blöcke frei", d.free_blocks()), atari: false, entries })
        } else {
            Err(format!("{name}: unbekanntes Imageformat (erwartet .d64, .d71, .d81 oder .atr)"))
        }
    }

    /// Dateiname im Projekt: Kleinbuchstaben, nur Buchstaben/Ziffern/`._-`; Commodore-Dateien bekommen die Endung der Dateiart.
    pub fn suggested_name(&self, e: &ImageEntry) -> String {
        let mut s: String = e.name.chars().map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c.to_ascii_lowercase() } else { '_' }).collect();
        s = s.trim_start_matches('.').to_string();
        if s.is_empty() { s = "datei".into(); }
        if !self.atari {
            let ext = match e.kind.as_str() { "PRG" => ".prg", "SEQ" => ".seq", "USR" => ".usr", "REL" => ".rel", _ => ".bin" };
            s.push_str(ext);
        }
        s
    }

    /// Schreibt eine Datei des Images nach `root/dir/<name>` (siehe [`write_unique`]).
    pub fn extract(&self, idx: usize, root: &Path, dir: &str, name: &str) -> Result<String, String> {
        let e = self.entries.get(idx).ok_or("Eintrag nicht vorhanden")?;
        let data = e.data.as_ref().map_err(|m| format!("{}: {m}", e.name))?;
        write_unique(root, dir, name, data)
    }
}

/// Schreibt `data` nach `root/dir/<name>`; gibt es den Namen schon, wird `_2`, `_3` … vor die Endung gesetzt (nichts wird überschrieben).
/// Rückgabe: relativer Pfad. `dir` und `name` dürfen den Projektordner nicht verlassen.
pub fn write_unique(root: &Path, dir: &str, name: &str, data: &[u8]) -> Result<String, String> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.starts_with('.') { return Err(format!("ungültiger Dateiname »{name}«")); }
    if dir.split('/').any(|p| p == "..") || dir.starts_with('/') || dir.contains('\\') { return Err(format!("ungültiger Zielordner »{dir}«")); }
    let folder = if dir.is_empty() { root.to_path_buf() } else { root.join(dir) };
    std::fs::create_dir_all(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let (stem, ext) = match name.rfind('.') { Some(i) => (&name[..i], &name[i..]), None => (name, "") };
    let mut final_name = name.to_string(); let mut n = 2;
    while folder.join(&final_name).exists() { final_name = format!("{stem}_{n}{ext}"); n += 1; }
    std::fs::write(folder.join(&final_name), data).map_err(|e| format!("{final_name}: {e}"))?;
    Ok(if dir.is_empty() { final_name } else { format!("{dir}/{final_name}") })
}

// ---------------------------------------------------------------------------------------------------------------------
// BASIC-Erkennung und Übernahme ins Projekt

/// Ein erkanntes BASIC-Programm (tokenisiert)
pub struct BasicInfo {
    /// Dialekt-ID (`cbm_v2`, `basic7`, `basic35`, `basic65`, `atari_basic`)
    pub dialect: String,
    /// Passendes System (`c64`, `c128`, `plus4`, `mega65`, `atari8`)
    pub system: &'static str,
    pub lines: usize,
    /// Bytes hinter dem BASIC-Programm (Maschinencode-Anhang); 0 = reines BASIC
    pub tail: usize,
    pub atari: bool,
}

/// Atari-Binärdatei (XEX/COM): Endung oder $FFFF am Anfang
pub fn is_xex(name: &str, b: &[u8]) -> bool {
    let n = name.to_ascii_lowercase();
    n.ends_with(".xex") || n.ends_with(".com") || (b.len() > 6 && b[0] == 0xFF && b[1] == 0xFF && !n.ends_with(".prg"))
}

/// Erkennt tokenisiertes BASIC: Commodore-PRG mit gültiger Zeilenverkettung an einer BASIC-Startadresse
/// ($0801 C64, $1C01 C128, $1001 Plus/4, $2001 MEGA65, $1201/$0401 VC-20) bzw. Atari-BASIC-SAVE-Datei (nur wenn `atari`).
pub fn detect_basic(data_dir: &Path, bytes: &[u8], atari: bool) -> Option<BasicInfo> {
    if atari {
        if is_xex("", bytes) { return None; }
        let d = vc_basic::atari::AtariDialect::load(data_dir, "atari_basic").ok()?;
        let p = vc_basic::atari::AtariProgram::parse_save(bytes, &d).ok()?;
        let lines = p.listing(&d).lines().count();
        return if lines > 0 { Some(BasicInfo { dialect: "atari_basic".into(), system: "atari8", lines, tail: 0, atari: true }) } else { None };
    }
    if bytes.len() < 6 { return None; }
    let (dialect, system) = match (bytes[0] as u16) | (bytes[1] as u16) << 8 {
        0x0801 => ("cbm_v2", "c64"), 0x1C01 => ("basic7", "c128"), 0x1001 => ("basic35", "plus4"), 0x2001 => ("basic65", "mega65"),
        // VC-20 (BASIC V2): +8 KB und mehr ab $1201, mit +3 KB ab $0401; $1001 teilen sich Grundgerät und Plus/4 (als Plus/4 gedeutet, die Tokens $80–$CB sind gleich)
        0x1201 | 0x0401 => ("cbm_v2", "vic20"),
        _ => return None,
    };
    let p = vc_basic::parse_prg(bytes).ok()?;
    if p.lines.is_empty() { return None; }
    // Programmlänge: Ladeadresse (2) + je Zeile Link(2)+Nummer(2)+Text+0 + Endmarke (2)
    let used = 2 + p.lines.iter().map(|(_, l)| 5 + l.len()).sum::<usize>() + 2;
    Some(BasicInfo { dialect: dialect.into(), system, lines: p.lines.len(), tail: bytes.len().saturating_sub(used), atari: false })
}

/// Listing als nummerierter Quelltext (`.bas`)
pub fn basic_listing(data_dir: &Path, bytes: &[u8], info: &BasicInfo) -> Result<String, String> {
    if info.atari {
        let d = vc_basic::atari::AtariDialect::load(data_dir, &info.dialect)?;
        Ok(vc_basic::atari::AtariProgram::parse_save(bytes, &d)?.listing(&d))
    } else {
        let d = vc_basic::Dialect::load(data_dir, &info.dialect)?;
        Ok(vc_basic::to_classic_text(&vc_basic::parse_prg(bytes)?, &d))
    }
}

/// Assembler-Quelltext aus einer PRG- oder XEX-Datei: `.org` je Block, darunter die Disassemblierung (Daten werden dabei als Befehle gedeutet).
pub fn asm_source(cpu: &vc_target::Cpu, name: &str, bytes: &[u8]) -> Result<String, String> {
    let mut out = format!("; aus {name} disassembliert (Daten können als Befehle erscheinen – bitte prüfen)\n");
    let blocks: Vec<(u16, Vec<u8>)> = if is_xex(name, bytes) { vc_disk::xex_blocks(bytes)?.into_iter().map(|(a, d)| (a, d.to_vec())).collect() }
        else if bytes.len() > 2 { vec![(bytes[0] as u16 | (bytes[1] as u16) << 8, bytes[2..].to_vec())] } else { return Err("Datei zu kurz".into()) };
    for (addr, data) in blocks {
        out.push_str(&format!(".org ${addr:04x}\n"));
        for l in vc_asm::disassemble(cpu, &data, addr) { out.push_str(&format!("    {}\n", l.text)); }
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub struct ImportOpts {
    /// Zielordner im Projekt ("" = Projektordner)
    pub dir: String,
    /// BASIC als Quelltext (`.bas`) übernehmen, wenn erkannt
    pub basic: bool,
    /// Originaldatei zusätzlich behalten (bei BASIC mit Maschinencode-Anhang immer)
    pub raw: bool,
    /// zusätzlich Disassembler-Quelltext (`.asm`) für Nicht-BASIC-Programme
    pub asm: bool,
    /// Prozessor für `.asm` (Standard: 6510 bzw. Sally)
    pub cpu: Option<String>,
}
impl Default for ImportOpts { fn default() -> Self { ImportOpts { dir: "import".into(), basic: true, raw: false, asm: false, cpu: None } } }

#[derive(Default, Debug)]
pub struct Outcome { pub files: Vec<String>, pub notes: Vec<String> }

fn stem_of(name: &str) -> &str { name.rfind('.').map_or(name, |i| &name[..i]) }

/// Übernimmt eine Datei ins Projekt. `file_name` ist der Name der Originaldatei im Projekt; `is_program` = PRG/XEX/DOS-Datei (nicht SEQ/Text).
/// BASIC wird (wenn `o.basic`) als `<name>.bas` abgelegt; die Originaldatei bleibt dann nur mit `o.raw` oder bei Maschinencode-Anhang erhalten
/// (Atari-BASIC-Originale heißen dann `<name>.tok`, damit sie nicht mit dem Quelltext verwechselt werden).
pub fn import_data(root: &Path, file_name: &str, data: &[u8], is_program: bool, atari: bool, data_dir: Option<&Path>, o: &ImportOpts) -> Result<Outcome, String> {
    let mut out = Outcome::default();
    let info = if is_program && o.basic { data_dir.and_then(|d| detect_basic(d, data, atari)) } else { None };
    if let (Some(b), Some(dd)) = (&info, data_dir) {
        let text = basic_listing(dd, data, b)?;
        let rel = write_unique(root, &o.dir, &format!("{}.bas", stem_of(file_name)), text.as_bytes())?;
        out.notes.push(format!("BASIC ({}, {} Zeilen) als Quelltext übernommen", b.dialect, b.lines));
        out.files.push(rel);
        if b.tail > 0 { out.notes.push(format!("Hinter dem BASIC-Programm folgen {} Bytes (Maschinencode?): Original bleibt erhalten", b.tail)); }
        if o.raw || b.tail > 0 {
            let raw_name = if b.atari { format!("{}.tok", stem_of(file_name)) } else { file_name.to_string() };
            out.files.push(write_unique(root, &o.dir, &raw_name, data)?);
        }
        return Ok(out);
    }
    out.files.push(write_unique(root, &o.dir, file_name, data)?);
    if o.asm && is_program {
        let dd = data_dir.ok_or("Hardware-Datenbank (data/) nicht gefunden")?;
        let cpu = vc_target::load_cpu(dd, o.cpu.as_deref().unwrap_or(if atari { "sally" } else { "6510" }))?;
        let text = asm_source(&cpu, file_name, data)?;
        let first = out.files[0].clone();
        let (dir, f) = first.rsplit_once('/').unwrap_or(("", &first));
        out.files.push(write_unique(root, dir, &format!("{}.asm", stem_of(f)), text.as_bytes())?);
        out.notes.push("Quelltext durch Disassemblieren erzeugt (Daten können als Befehle erscheinen)".into());
    }
    Ok(out)
}
