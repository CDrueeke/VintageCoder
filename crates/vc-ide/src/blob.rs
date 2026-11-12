// SPDX-License-Identifier: GPL-3.0-or-later
//! BASIC mit Maschinencode: `blob "player.asm" with "song.asm"` legt den Code **hinter den BASIC-Text** ins PRG.
//! Zweistufiger Bau: Die Adresse des Codes hängt von der Länge des BASIC-Teils ab, und dessen Länge von den Zahlen,
//! die der Übersetzer für die Einsprungadressen einsetzt. Darum wird wiederholt, bis die Adresse stabil ist.
use std::path::{Path, PathBuf};
use vc_asm::{assemble, Options as AsmOptions, Policy};
use vc_basic::atari::AtariDialect;
use vc_basic::{compile, compile_atari, AtariCompiled, AtariOptions, BlobReq, Compiled, Dialect, Options};
use vc_core::Diagnostic;

pub struct BlobInfo { pub file: String, pub addr: u16, pub size: usize, pub symbols: usize, /// Marken des Codes (Name klein, Wert) – für Tests und Debugger
    pub syms: Vec<(String, i64)> }
pub struct Built { pub compiled: Compiled, pub prg: Vec<u8>, pub blobs: Vec<BlobInfo> }

fn find(req: &BlobReq, name: &str, root: &Path) -> Option<PathBuf> {
    let base = Path::new(&req.src.0).parent().map(Path::to_path_buf).unwrap_or_default();
    [base, root.to_path_buf()].iter().map(|d| d.join(name)).find(|p| p.is_file())
}

fn diag(req: &BlobReq, msg: impl Into<String>) -> Vec<Diagnostic> { vec![Diagnostic::error("B114", &req.src.0, req.src.1, msg)] }

/// Übersetzt alle Blobs ab `addr`; Ergebnis: Bytes, Konstanten (Symbole) und Angaben je Blob.
type Assembled = (Vec<u8>, Vec<(String, i64)>, Vec<BlobInfo>, Vec<(u16, Vec<u8>)>);

/// `strict`: der Code muss mit `.org blob` beginnen (Commodore); sonst darf er hinter `blob` liegen (Atari, Tabellen vor dem Player)
fn assemble_blobs(reqs: &[BlobReq], mut addr: usize, system: &str, root: &Path, data: &Path, tgt: &vc_target::Target, strict: bool) -> Result<Assembled, Vec<Diagnostic>> {
    let cpu = vc_target::load_cpu(data, &tgt.cpu).map_err(|e| diag(&reqs[0], e))?;
    let chips = vc_target::chip_symbols(data, tgt).map_err(|e| diag(&reqs[0], e))?;
    let (mut all, mut consts, mut infos, mut chunks) = (vec![], vec![], vec![], vec![]);
    for r in reqs {
        let name = r.file.replace("{system}", system);
        let path = find(r, &name, root).ok_or_else(|| diag(r, format!("blob: Datei '{name}' nicht gefunden")))?;
        let mut text = std::fs::read_to_string(&path).map_err(|e| diag(r, format!("{}: {e}", path.display())))?;
        if let Some(w) = r.with.first() {
            let w = w.replace("{system}", system);
            let stem = Path::new(&w).file_stem().and_then(|s| s.to_str()).unwrap_or("song");
            text = text.replace("{song}", stem).replace("{song_file}", &w);
        }
        let mut o = AsmOptions::new(path.to_string_lossy(), Policy::Warn);
        o.include_dirs.push(root.to_path_buf());
        o.include_dirs.push(data.join("sys"));
        o.atari_text = tgt.text_encoding == "atari";
        o.basic_start = tgt.basic_load_addr;
        o.predefined = chips.clone();
        o.predefined.push(("blob".into(), addr as i64));
        let out = assemble(&text, &cpu, &o)?;
        let mut segs: Vec<_> = out.segments.iter().filter(|s| !s.bytes.is_empty()).collect();
        segs.sort_by_key(|s| s.start);
        let Some(first) = segs.first() else { return Err(diag(r, format!("blob '{name}' erzeugt keinen Code"))) };
        if (strict && first.start as usize != addr) || (first.start as usize) < addr { return Err(diag(r, format!("blob '{name}' muss mit `.org blob` beginnen (beginnt bei ${:04X}, erwartet ${addr:04X})", first.start))); }
        let mut bytes: Vec<u8> = vec![];
        for s in &segs {
            chunks.push((s.start, s.bytes.clone()));
            let at = s.start as usize - addr;
            if at < bytes.len() { return Err(diag(r, format!("blob '{name}': überlappende Segmente"))); }
            bytes.resize(at, 0);
            bytes.extend_from_slice(&s.bytes);
        }
        let n = out.symbols.iter().filter(|(k, _)| !k.contains('.') && !k.starts_with('@') && !k.starts_with('_')).count();
        let mine: Vec<(String, i64)> = out.symbols.iter().filter(|(k, _)| !k.contains('.') && !k.starts_with('@') && !k.starts_with('_') && k.as_str() != "blob").map(|(k, v)| (k.to_ascii_lowercase(), *v)).collect();
        consts.extend(mine.iter().cloned());
        infos.push(BlobInfo { file: name, addr: addr as u16, size: bytes.len(), symbols: n, syms: mine });
        addr += bytes.len();
        all.extend(bytes);
    }
    Ok((all, consts, infos, chunks))
}

struct Blobs { blobs: Vec<BlobReq> }

/// Systeme, bei denen BASIC Code hinter dem BASIC-Text nicht vor den Variablen schützt: der Code kommt als DATA-Zeilen und wird an diese
/// feste Adresse (Bank 0) geladen. MEGA65: BASIC 65 überschreibt den Code (auf Xemu mit der Original-ROM gemessen).
/// Gemessen (Xemu, BASIC 65 V920395, 2026-11-06): `POKE`/`PEEK` gehen nur nach `BANK 0` in Bank 0 (ohne `BANK` und mit `BANK 128` liest `$3000` immer 48), das Laden braucht also `BANK 0`.
/// Felder (A(400), B%(800), A$(60)) und Zeichenketten überschreiben je 256 Byte ab `$3000`, `$4000`, `$6000`, `$8000`, `$A000`, `$C000` nicht; größere Blöcke und `$D000` ff. (E/A, ROM) sind nicht geprüft.
pub const MEGA65_BLOB_BASE: usize = 0xC000;
fn chunk_base(system: &str) -> Option<usize> { (system == "mega65").then_some(MEGA65_BLOB_BASE) }

/// Übersetzt `.sbas`; enthält der Text `blob`-Zeilen, wird der Code hinter den BASIC-Text gelegt (oder, wo das nicht geht, als DATA-Zeilen mitgeliefert).
pub fn compile_with_blobs(src: &str, d: &Dialect, o: &Options, root: &Path, data: &Path, tgt: &vc_target::Target) -> Result<Built, Vec<Diagnostic>> {
    let system = o.system.clone().unwrap_or_else(|| "c64".into());
    let reqs = vc_basic::find_blobs(src, &o.file, &o.include_dirs, Some(&system), &o.consts)?;
    if reqs.is_empty() { let c = compile(src, d, o)?; return Ok(Built { prg: c.prg.clone(), compiled: c, blobs: vec![] }); }
    if let Some(base) = chunk_base(&system) {
        let (_bytes, mut consts, infos, chunks) = assemble_blobs(&reqs, base, &system, root, data, tgt, false)?;
        consts.push(("blob_addr".into(), base as i64));
        let mut o2 = o.clone();
        o2.consts.extend(consts);
        o2.blob_chunks = chunks;
        let c = compile(src, d, &o2)?;
        return Ok(Built { prg: c.prg.clone(), compiled: c, blobs: infos });
    }
    let first = Blobs { blobs: reqs };
    let load = o.load_addr.unwrap_or(d.load_addr) as usize;
    let mut o1 = o.clone();
    o1.blob_chunks.clear();
    let first_c = compile(src, d, &o1)?;
    let mut addr = load + first_c.prg.len() - 2;
    for _ in 0..8 {
        let (bytes, consts, infos, _) = assemble_blobs(&first.blobs, addr, &system, root, data, tgt, true)?;
        let mut o2 = o.clone();
        o2.consts.extend(consts);
        let c = compile(src, d, &o2)?;
        let end = load + c.prg.len() - 2;
        if end == addr {
            if end + bytes.len() > 0xFFFF { return Err(diag(&first.blobs[0], format!("Programm samt Maschinencode zu groß (endet bei ${:05X})", end + bytes.len()))); }
            let mut prg = c.prg.clone();
            prg.extend_from_slice(&bytes);
            return Ok(Built { compiled: c, prg, blobs: infos });
        }
        addr = end;
    }
    Err(diag(&first.blobs[0], "Adresse des Maschinencodes wird nicht stabil"))
}

pub struct BuiltAtari { pub compiled: AtariCompiled, pub blobs: Vec<BlobInfo> }

/// Atari BASIC kann keinen Maschinencode hinter dem Programm tragen: der Code wird oben im Speicher (unter dem BASIC-ROM, ab einer Seitengrenze)
/// angelegt und als DATA-Zeilen mitgeliefert (`@blob_data`, siehe `AtariOptions::blob_chunks`). Dem Programm stehen `blob_addr`, `blob_page`
/// (für `poke 106`) und alle Marken des Codes als Konstanten zur Verfügung.
pub fn compile_atari_with_blobs(src: &str, d: &AtariDialect, o: &AtariOptions, root: &Path, data: &Path, tgt: &vc_target::Target) -> Result<BuiltAtari, Vec<Diagnostic>> {
    let system = o.system.clone().unwrap_or_else(|| "atari8".into());
    let reqs = vc_basic::find_blobs(src, &o.file, &o.include_dirs, Some(&system), &o.consts)?;
    if reqs.is_empty() { return Ok(BuiltAtari { compiled: compile_atari(src, d, o)?, blobs: vec![] }); }
    let first = Blobs { blobs: reqs };
    // Größe messen (der Code hängt nicht von der Adresse ab), dann die Basisadresse so wählen, dass der Code unter $A000 endet
    let (probe, ..) = assemble_blobs(&first.blobs, 0x6000, &system, root, data, tgt, false)?;
    let base = (0xA000usize.saturating_sub(probe.len())) & 0xFF00;
    if base < 0x4000 { return Err(diag(&first.blobs[0], format!("Maschinencode zu groß ({} Bytes)", probe.len()))); }
    let (_bytes, mut consts, infos, chunks) = assemble_blobs(&first.blobs, base, &system, root, data, tgt, false)?;
    consts.push(("blob_addr".into(), base as i64));
    consts.push(("blob_page".into(), (base >> 8) as i64));
    let mut o2 = o.clone();
    o2.consts.extend(consts);
    o2.blob_chunks = chunks;
    let compiled = compile_atari(src, d, &o2)?;
    Ok(BuiltAtari { compiled, blobs: infos })
}
