// SPDX-License-Identifier: GPL-3.0-or-later
//! PNG-Dekoder gegen fremde Eingaben: Dekompressionsbombe, zu kurzer IHDR, Prüfsumme, Größe.
use vc_media::png;

struct BitW { out: Vec<u8>, cur: u32, n: u32 }
impl BitW {
    fn bits(&mut self, v: u32, n: u32) { for i in 0..n { self.cur |= ((v >> i) & 1) << self.n; self.n += 1; if self.n == 8 { self.out.push(self.cur as u8); self.cur = 0; self.n = 0; } } }
    fn code(&mut self, c: u32, n: u32) { for i in (0..n).rev() { self.bits((c >> i) & 1, 1); } }   // Huffman-Codes: höchstes Bit zuerst
    fn fertig(mut self) -> Vec<u8> { if self.n > 0 { self.out.push(self.cur as u8); } self.out }
}

/// zlib-Strom (feste Huffman-Codes), der `wiederholungen` × 258 Nullbytes ergibt: 1 Byte Eingabe je ~258 Bytes Ausgabe
fn bombe(wiederholungen: usize) -> Vec<u8> {
    let mut w = BitW { out: vec![0x78, 0x01], cur: 0, n: 0 };
    w.bits(1, 1); w.bits(1, 2);
    w.code(0x30, 8);                          // Literal 0
    for _ in 0..wiederholungen { w.code(0xC0 + 5, 8); w.code(0, 5); }   // Länge 258, Abstand 1
    w.code(0, 7);                             // Ende
    let mut z = w.fertig(); z.extend_from_slice(&[0, 0, 0, 0]); z
}

fn crc(d: &[u8]) -> u32 { let mut c = !0u32; for &b in d { c ^= b as u32; for _ in 0..8 { c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 }; } } !c }
fn chunk(out: &mut Vec<u8>, t: &[u8; 4], d: &[u8]) { out.extend_from_slice(&(d.len() as u32).to_be_bytes()); let mut td = t.to_vec(); td.extend_from_slice(d); out.extend_from_slice(&td); out.extend_from_slice(&crc(&td).to_be_bytes()); }
fn png_mit(ihdr: &[u8], idat: &[u8]) -> Vec<u8> { let mut o = b"\x89PNG\r\n\x1a\n".to_vec(); chunk(&mut o, b"IHDR", ihdr); chunk(&mut o, b"IDAT", idat); chunk(&mut o, b"IEND", &[]); o }
fn ihdr(w: u32, h: u32) -> Vec<u8> { let mut v = w.to_be_bytes().to_vec(); v.extend_from_slice(&h.to_be_bytes()); v.extend_from_slice(&[8, 6, 0, 0, 0]); v }

#[test]
fn bombe_wird_mit_grenze_abgebrochen() {
    let z = bombe(40_000);   // ~10 MB Ausgabe aus ~65 KB
    assert!(z.len() < 70_000);
    assert_eq!(png::inflate_max(&z, 20_000_000).unwrap().len(), 40_000 * 258 + 1);
    let e = png::inflate_max(&z, 1_000_000).unwrap_err();
    assert!(e.contains("zu groß") || e.contains("too large"), "{e}");
    // das Bild darf nicht mehr Daten haben, als seine Größe verlangt
    let e = png::decode(&png_mit(&ihdr(4, 4), &z)).err().unwrap();
    assert!(e.contains("zu groß") || e.contains("too large"), "{e}");
}

#[test]
fn ihdr_zu_kurz_ist_ein_fehler_keine_panik() {
    for n in 0..13 { let e = png::decode(&png_mit(&vec![0u8; n], &[])).err().unwrap(); assert!(!e.is_empty()); }
}

#[test]
fn abgeschnitten_und_riesige_blocklaenge_ohne_panik() {
    let mut o = b"\x89PNG\r\n\x1a\n".to_vec();
    o.extend_from_slice(&u32::MAX.to_be_bytes()); o.extend_from_slice(b"IHDR"); o.extend_from_slice(&[0; 20]);
    assert!(png::decode(&o).is_err());
}

#[test]
fn pruefsumme_wird_geprueft() {
    let rgba = vec![200u8; 4 * 4 * 4];
    let gut = png::encode_rgba(4, 4, &rgba);
    assert_eq!(png::decode(&gut).unwrap().rgba, rgba);
    let mut schlecht = gut.clone(); schlecht[8 + 8 + 2] ^= 1;   // Bit im IHDR
    let e = png::decode(&schlecht).err().unwrap(); assert!(e.contains("IHDR"), "{e}");
    let mut schlecht = gut.clone(); let n = schlecht.len(); schlecht[n - 12 - 6] ^= 1;   // Bit in IDAT
    assert!(png::decode(&schlecht).is_err());
}

#[test]
fn bildgroesse_ist_begrenzt() {
    let e = png::decode(&png_mit(&ihdr(8192, 8192), &[0x78, 1])).err().unwrap(); assert!(!e.is_empty());
    assert!(png::decode(&png::encode_rgba(1, 1, &[1, 2, 3, 4])).is_ok());
}
