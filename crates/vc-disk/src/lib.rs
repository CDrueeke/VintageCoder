// SPDX-License-Identifier: GPL-3.0-or-later
//! Disk-Images: ATR-Container, Atari-DOS-2.0S-Dateisystem (90K), bootbare XEX-Disketten, CBM-DOS-Images D64, D71 und D81.
mod any;
mod atr;
mod boot;
mod cbm;
mod d64;

pub use any::{Addr, AnyDisk, Analysis, Cell, FileInfo, Format};
pub use atr::{Atr, Dos2Entry};
pub use boot::{boot_disk, xex_blocks, xex_segments};
pub use cbm::{Disk, Kind};
pub use d64::{sectors_in_track, D64, D64Entry};

/// Kürzt eine lange Problemliste: die ersten `max` Meldungen und eine Sammelzeile für den Rest
pub fn summarize_problems(problems: &[String], max: usize) -> Vec<String> {
    if problems.len() <= max { return problems.to_vec(); }
    let mut v: Vec<String> = problems[..max].to_vec();
    v.push(format!("… und {} weitere Probleme", problems.len() - max));
    v
}
