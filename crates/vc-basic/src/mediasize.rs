// SPDX-License-Identifier: GPL-3.0-or-later
//! Platzbedarf und Lesereihenfolge bei eingebundenen Medien (Konzept „Include von Medien“, 3.5 und 3.7).
//!
//! * `B195`: das übersetzte Programm ist größer als der freie BASIC-Speicher des Zielsystems (Meldung des Baus statt erst beim Laden).
//! * `B196`: mehrere Medien eingebunden, ein `RESTORE @marke` wird benutzt, aber der Dialekt kennt kein `RESTORE` auf eine Zeile (CBM V2): `READ` liest
//!   streng der Reihe nach, die Medien müssen in Lesereihenfolge eingebunden sein.
use vc_core::{msg, Diagnostic};

/// Freier BASIC-Speicher in Byte beim Einschalten (Meldung „BYTES FREE“) für das Zielsystem, soweit eindeutig:
/// C64 38911, VC-20 ohne Erweiterung 3583, Plus/4 60671. Andere Systeme: `None` (keine Prüfung).
pub fn free_bytes(system: &str) -> Option<u32> {
    match system { "c64" => Some(38911), "vic20" => Some(3583), "plus4" => Some(60671), _ => None }
}

/// Warnung `B195`, wenn `size` (Bytes des Programms ohne Ladeadresse) nicht in den freien Speicher passt
pub fn check_size(size: usize, system: Option<&str>, file: &str) -> Option<Diagnostic> {
    let sys = system?;
    let free = free_bytes(sys)?;
    if size as u64 <= free as u64 { return None; }
    Some(Diagnostic::warning("B195", file, 1, msg!("diag-basic-b195-program-too-big", size = size, free = free, system = sys)))
}

/// Benutzt der Text `RESTORE @name` (außerhalb von `REM`)?
fn uses_restore_label(src: &str) -> Option<usize> {
    for (i, l) in src.lines().enumerate() {
        let u = l.to_ascii_uppercase();
        let code = match u.find("REM") { Some(p) if !u[..p].contains('"') => &u[..p], _ => &u[..] };
        let mut from = 0;
        while let Some(p) = code[from..].find("RESTORE") {
            let after = code[from + p + 7..].trim_start();
            if after.starts_with('@') { return Some(i + 1); }
            from += p + 7;
        }
    }
    None
}

/// Warnung `B196` (nur Dialekte ohne `RESTORE`-Sprungziel: `jumps` ohne `RESTORE`)
pub fn check_order(src: &str, media: usize, jumps: &[String], dialect: &str, file: &str) -> Option<Diagnostic> {
    if media < 2 || jumps.iter().any(|j| j == "RESTORE") { return None; }
    let line = uses_restore_label(src)?;
    Some(Diagnostic::warning("B196", file, line, msg!("diag-basic-b196-media-restore-order", count = media, dialect = dialect)))
}
