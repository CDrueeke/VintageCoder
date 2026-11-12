// SPDX-License-Identifier: GPL-3.0-or-later
//! Eingebaute **Ersatzschrift** für Zeichensätze, wenn kein Zeichensatz-ROM des Anwenders vorliegt. Eigenbau von VintageCoder (5×7-Zeichen in 8×8-Zellen,
//! Buchstaben, Ziffern und Satzzeichen; keine Grafikzeichen, Kleinbuchstaben sehen aus wie Großbuchstaben). Sie hat mit den Original-Zeichensätzen
//! der Systeme nichts zu tun und steht unter der Lizenz von VintageCoder (GPL-3.0-oder-später). Sie ist ausdrücklich ein Notbehelf, kein Ersatz für die Originalschrift.
use crate::systems::SetDef;

/// Zeichen in der Form `=<Zeichen>` gefolgt von sieben Zeilen zu je fünf Stellen (`#` gesetzt, `.` leer)
const ART: &str = "
=A
.###.
#...#
#...#
#####
#...#
#...#
#...#
=B
####.
#...#
#...#
####.
#...#
#...#
####.
=C
.###.
#...#
#....
#....
#....
#...#
.###.
=D
####.
#...#
#...#
#...#
#...#
#...#
####.
=E
#####
#....
#....
####.
#....
#....
#####
=F
#####
#....
#....
####.
#....
#....
#....
=G
.###.
#...#
#....
#.###
#...#
#...#
.####
=H
#...#
#...#
#...#
#####
#...#
#...#
#...#
=I
.###.
..#..
..#..
..#..
..#..
..#..
.###.
=J
..###
...#.
...#.
...#.
...#.
#..#.
.##..
=K
#...#
#..#.
#.#..
##...
#.#..
#..#.
#...#
=L
#....
#....
#....
#....
#....
#....
#####
=M
#...#
##.##
#.#.#
#.#.#
#...#
#...#
#...#
=N
#...#
##..#
#.#.#
#..##
#...#
#...#
#...#
=O
.###.
#...#
#...#
#...#
#...#
#...#
.###.
=P
####.
#...#
#...#
####.
#....
#....
#....
=Q
.###.
#...#
#...#
#...#
#.#.#
#..#.
.##.#
=R
####.
#...#
#...#
####.
#.#..
#..#.
#...#
=S
.####
#....
#....
.###.
....#
....#
####.
=T
#####
..#..
..#..
..#..
..#..
..#..
..#..
=U
#...#
#...#
#...#
#...#
#...#
#...#
.###.
=V
#...#
#...#
#...#
#...#
#...#
.#.#.
..#..
=W
#...#
#...#
#...#
#.#.#
#.#.#
##.##
#...#
=X
#...#
#...#
.#.#.
..#..
.#.#.
#...#
#...#
=Y
#...#
#...#
.#.#.
..#..
..#..
..#..
..#..
=Z
#####
....#
...#.
..#..
.#...
#....
#####
=0
.###.
#...#
#..##
#.#.#
##..#
#...#
.###.
=1
..#..
.##..
..#..
..#..
..#..
..#..
.###.
=2
.###.
#...#
....#
...#.
..#..
.#...
#####
=3
#####
...#.
..#..
...#.
....#
#...#
.###.
=4
...#.
..##.
.#.#.
#..#.
#####
...#.
...#.
=5
#####
#....
####.
....#
....#
#...#
.###.
=6
..##.
.#...
#....
####.
#...#
#...#
.###.
=7
#####
....#
...#.
..#..
.#...
.#...
.#...
=8
.###.
#...#
#...#
.###.
#...#
#...#
.###.
=9
.###.
#...#
#...#
.####
....#
...#.
.##..
=!
..#..
..#..
..#..
..#..
..#..
.....
..#..
=\"
.#.#.
.#.#.
.....
.....
.....
.....
.....
=#
.#.#.
#####
.#.#.
.#.#.
#####
.#.#.
.....
=$
..#..
.####
#.#..
.###.
..#.#
####.
..#..
=%
##..#
##.#.
...#.
..#..
.#...
.#.##
#..##
=&
.##..
#..#.
#.#..
.#...
#.#.#
#..#.
.##.#
='
..#..
..#..
.#...
.....
.....
.....
.....
=(
...#.
..#..
.#...
.#...
.#...
..#..
...#.
=)
.#...
..#..
...#.
...#.
...#.
..#..
.#...
=*
.....
#.#.#
.###.
#####
.###.
#.#.#
.....
=+
.....
..#..
..#..
#####
..#..
..#..
.....
=,
.....
.....
.....
.....
..##.
...#.
..#..
=-
.....
.....
.....
#####
.....
.....
.....
=.
.....
.....
.....
.....
.....
..##.
..##.
=/
....#
...#.
...#.
..#..
.#...
.#...
#....
=:
.....
..##.
..##.
.....
..##.
..##.
.....
=;
.....
..##.
..##.
.....
..##.
...#.
..#..
=<
...#.
..#..
.#...
#....
.#...
..#..
...#.
==
.....
.....
#####
.....
#####
.....
.....
=>
.#...
..#..
...#.
....#
...#.
..#..
.#...
=?
.###.
#...#
....#
...#.
..#..
.....
..#..
=@
.###.
#...#
#.###
#.#.#
#.###
#....
.###.
=[
.###.
.#...
.#...
.#...
.#...
.#...
.###.
=]
.###.
...#.
...#.
...#.
...#.
...#.
.###.
=£
..##.
.#..#
.#...
####.
.#...
.#...
#####
=^
..#..
.#.#.
#...#
.....
.....
.....
.....
=_
.....
.....
.....
.....
.....
.....
#####
=\\
#....
.#...
.#...
..#..
...#.
...#.
....#
=↑
..#..
.###.
#.#.#
..#..
..#..
..#..
..#..
=←
.....
..#..
.#...
#####
.#...
..#..
.....
";

/// Zeichen → acht Zeilen (letzte Zeile leer), Bit 7 ganz links: Spalten 1 bis 5 der Zelle
fn glyph(c: char) -> [u8; 8] {
    let mut cur: Option<char> = None;
    let mut rows: Vec<u8> = vec![];
    for line in ART.lines() {
        if let Some(rest) = line.strip_prefix('=') {
            if cur == Some(c) && rows.len() == 7 { break; }
            cur = rest.chars().next().map(|x| if rest == "\\\"" { '"' } else if rest == "\\\\" { '\\' } else { x });
            rows.clear();
            continue;
        }
        if cur == Some(c) && !line.is_empty() {
            let bits = line.chars().take(5).enumerate().fold(0u8, |a, (i, ch)| if ch == '#' { a | (0x40 >> i) } else { a });
            rows.push(bits);
        }
    }
    let mut out = [0u8; 8];
    if rows.len() == 7 { out[..7].copy_from_slice(&rows); }
    out
}

/// Bildschirmcode-Reihenfolge der Commodore-Systeme: `@ A–Z [ £ ] ↑ ← Leerzeichen ! " # … ?`
const CBM_ORDER: &str = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ[£]↑← !\"#$%&'()*+,-./0123456789:;<=>?";
/// Atari-Bildschirmcode 0–63: ASCII 0x20–0x5F
const ATARI_ORDER: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_";

fn cbm_glyph(code: usize, upper_for_lower: bool) -> [u8; 8] {
    let _ = upper_for_lower;
    let (base, inverse) = (code & 0x7F, code >= 128);
    let mut g = if base < 64 { glyph(CBM_ORDER.chars().nth(base).unwrap_or(' ')) } else { [0; 8] };
    if inverse { for b in g.iter_mut() { *b = !*b; } }
    g
}

/// Bytes eines eingebauten Satzes in der Anordnung des Systems; `None`, wenn es für das Profil keine Ersatzschrift gibt
pub fn bytes_for(set: &SetDef) -> Option<Vec<u8>> {
    let p = set.profile.as_str();
    if p.ends_with(".vdc") {
        // 8×16: jede Zeile doppelt, 256 Zeichen
        return Some((0..256).flat_map(|c| cbm_glyph(c, true).into_iter().flat_map(|b| [b, b])).take(set.size).collect());
    }
    if p == "atari.char.antic2" {
        let mut v = Vec::with_capacity(1024);
        for code in 0..128usize {
            let g = match code { 0..=63 => glyph(ATARI_ORDER.chars().nth(code).unwrap()), 97..=122 => glyph((b'A' + (code - 97) as u8) as char), _ => [0; 8] };
            v.extend(g);
        }
        return Some(v);
    }
    if p.ends_with(".char.hires") && matches!(p, "c64.char.hires" | "vic20.char.hires" | "plus4.char.hires" | "c128.char.hires") {
        return Some((0..256).flat_map(|c| cbm_glyph(c, true)).take(set.size).collect());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(profile: &str, size: usize) -> SetDef { SetDef { name: "upper".into(), label: String::new(), offset: 0, size, profile: profile.into() } }

    #[test]
    fn alle_zeichen_der_tabelle_sind_vollstaendig() {
        for c in CBM_ORDER.chars().chain(ATARI_ORDER.chars()).filter(|c| *c != ' ') {
            assert!(glyph(c).iter().any(|b| *b != 0), "Zeichen {c:?} fehlt oder ist leer");
        }
        assert_eq!(glyph(' '), [0; 8]);
    }

    #[test]
    fn groessen_und_anordnung() {
        let c64 = bytes_for(&set("c64.char.hires", 2048)).unwrap();
        assert_eq!(c64.len(), 2048);
        assert_eq!(&c64[8..16], &glyph('A'), "Bildschirmcode 1 = A");
        assert_eq!(c64[128 * 8..129 * 8].iter().map(|b| !b).collect::<Vec<_>>(), c64[0..8].to_vec(), "Zeichen 128.. = invertiert");
        assert_eq!(bytes_for(&set("c128.char.vdc", 4096)).unwrap().len(), 4096);
        let atari = bytes_for(&set("atari.char.antic2", 1024)).unwrap();
        assert_eq!(&atari[33 * 8..34 * 8], &glyph('A'), "Atari: Code 33 = A");
        assert!(bytes_for(&set("mega65.char.fcm", 2048)).is_none());
    }
}
