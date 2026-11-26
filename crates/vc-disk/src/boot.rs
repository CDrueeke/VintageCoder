// SPDX-License-Identifier: GPL-3.0-or-later
//! Bootbare Diskette aus einer XEX-Datei: ein in Assembler geschriebener Loader (`data/boot/xex-boot.asm`) liest die Segmente sektorweise.
use crate::atr::{Atr, SECTORS, SECTOR_SIZE};
use std::path::Path;
use vc_asm::{assemble, to_bin, Options, Policy};

/// Zerlegt eine XEX-Datei in (Start, Ende)-Segmente.
pub fn xex_segments(x: &[u8]) -> Result<Vec<(u16, u16)>, String> {
    let (mut i, mut v) = (0usize, vec![]);
    while i < x.len() {
        if x.len() < i + 2 { return Err(vc_i18n::t!("disk-abgeschnittener-segmentkopf").into()); }
        if x[i] == 0xFF && x[i + 1] == 0xFF { i += 2; if i >= x.len() { break; } }
        if i + 4 > x.len() { return Err(vc_i18n::t!("disk-abgeschnittener-segmentkopf").into()); }
        let (s, e) = (x[i] as u16 | (x[i + 1] as u16) << 8, x[i + 2] as u16 | (x[i + 3] as u16) << 8);
        if e < s { return Err(vc_i18n::t!("disk-segment-s-e-ende-vor-start", s = format!("{:04X}", s), e = format!("{:04X}", e))); }
        let len = (e - s) as usize + 1;
        if i + 4 + len > x.len() { return Err(vc_i18n::t!("disk-segment-s-e-ist-abgeschnitten", s = format!("{:04X}", s), e = format!("{:04X}", e))); }
        v.push((s, e));
        i += 4 + len;
    }
    if v.is_empty() { Err(vc_i18n::t!("disk-keine-segmente").into()) } else { Ok(v) }
}

/// Zerlegt eine XEX-Datei in (Startadresse, Daten)-Blöcke (auch für Disassembler und Import).
pub fn xex_blocks(x: &[u8]) -> Result<Vec<(u16, &[u8])>, String> {
    let mut v = vec![];
    let mut off = 0usize;
    for (s, e) in xex_segments(x)? {
        // Segmentkopf (optional $FFFF) überspringen; xex_segments hat die Struktur schon geprüft
        if x[off] == 0xFF && x[off + 1] == 0xFF { off += 2; }
        off += 4; let len = (e - s) as usize + 1;
        v.push((s, &x[off..off + len])); off += len;
    }
    Ok(v)
}

/// Erzeugt eine bootbare 90K-ATR, die `xex` startet. Fehler, wenn Segmente den Loader ($0700–$08FF) überlappen.
pub fn boot_disk(xex: &[u8], data_dir: &Path) -> Result<Atr, String> {
    if xex.len() < 6 || xex[0] != 0xFF || xex[1] != 0xFF { return Err(vc_i18n::t!("disk-keine-xex-datei-ffff-kennung-fehlt").into()); }
    for (s, e) in xex_segments(xex)? {
        if s <= 0x08FF && e >= 0x0700 { return Err(vc_i18n::t!("disk-segment-s-e-ueberlappt-den-boot", s = format!("{:04X}", s), e = format!("{:04X}", e))); }
    }
    let first_data = 4usize;
    let max = (SECTORS - first_data + 1) * SECTOR_SIZE;
    if xex.len() > max { return Err(vc_i18n::t!("disk-xex-p1-bytes-passt-nicht-auf", p1 = xex.len(), max = &max)); }
    let path = data_dir.join("boot/xex-boot.asm");
    let src = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let cpu = vc_target::load_cpu(data_dir, "sally")?;
    let mut o = Options::new(path.to_string_lossy(), Policy::Deny);
    o.defines = vec![("XEX_SECTOR".into(), first_data as i64), ("XEX_LEN".into(), xex.len() as i64)];
    let out = assemble(&src, &cpu, &o).map_err(|d| d.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n"))?;
    let loader = to_bin(&out).map_err(|m| m.text())?;
    if loader.len() > 3 * SECTOR_SIZE { return Err(vc_i18n::t!("disk-boot-loader-ist-p1-bytes-gross", p1 = loader.len())); }
    if out.segments[0].start != 0x0700 || loader[1] != 3 { return Err(vc_i18n::t!("disk-unerwartetes-loader-layout").into()); }
    let mut a = Atr::blank();
    for (k, chunk) in loader.chunks(SECTOR_SIZE).enumerate() { a.sector_mut(1 + k)[..chunk.len()].copy_from_slice(chunk); }
    for (k, chunk) in xex.chunks(SECTOR_SIZE).enumerate() { a.sector_mut(first_data + k)[..chunk.len()].copy_from_slice(chunk); }
    Ok(a)
}
