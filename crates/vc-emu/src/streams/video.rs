// SPDX-License-Identifier: GPL-3.0-or-later
//! Video-Strom: 780 Byte je Paket = 12 Byte Kopf (alles little-endian) + 768 Byte Pixel (4 Zeilen zu 384, 4 Bit je Pixel,
//! niedriges Nibble = linkes Pixel). 68 Pakete je PAL-Bild (384 × 272). Hier nur der Kopf; Pixel liefert [`VideoPacket::pixels`] roh.
use super::PacketError;

pub const PACKET_LEN: usize = 780;
pub const HEADER_LEN: usize = 12;
pub const PAYLOAD_LEN: usize = 768;
pub const WIDTH: u16 = 384;
pub const LINES_PER_PACKET: u8 = 4;
/// Größte plausible Bildhöhe (PAL 272, NTSC 240; etwas Reserve)
pub const MAX_LINES: u16 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoHeader {
    pub seq: u16,
    pub frame: u16,
    /// Erste Zeile dieses Pakets (ohne das Bit „letztes Paket“)
    pub line: u16,
    /// Bit 15 der Zeilenangabe: letztes Paket des Bilds
    pub last: bool,
    pub pixels_per_line: u16,
    pub lines_per_packet: u8,
    pub bits_per_pixel: u8,
    pub encoding: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoPacket<'a> { pub header: VideoHeader, /// 768 Byte, zwei Pixel je Byte
    pub pixels: &'a [u8] }

/// Paket zerlegen und prüfen
pub fn parse(p: &[u8]) -> Result<VideoPacket<'_>, PacketError> {
    if p.len() != PACKET_LEN { return Err(PacketError::Length { got: p.len(), want: PACKET_LEN }); }
    let u16at = |i: usize| u16::from_le_bytes([p[i], p[i + 1]]);
    let raw_line = u16at(4);
    let h = VideoHeader { seq: u16at(0), frame: u16at(2), line: raw_line & 0x7FFF, last: raw_line & 0x8000 != 0, pixels_per_line: u16at(6), lines_per_packet: p[8], bits_per_pixel: p[9], encoding: u16at(10) };
    if h.pixels_per_line != WIDTH { return Err(PacketError::Header("Pixel je Zeile")); }
    if h.lines_per_packet != LINES_PER_PACKET { return Err(PacketError::Header("Zeilen je Paket")); }
    if h.bits_per_pixel != 4 { return Err(PacketError::Header("Bits je Pixel")); }
    if h.encoding != 0 { return Err(PacketError::Header("Kodierung")); }
    if h.line as u32 + h.lines_per_packet as u32 > MAX_LINES as u32 { return Err(PacketError::Header("Zeile außerhalb des Bilds")); }
    Ok(VideoPacket { header: h, pixels: &p[HEADER_LEN..HEADER_LEN + PAYLOAD_LEN] })
}

/// Pepto-Palette (C64, `data/palettes/c64.json`) als RGB; die Ultimate sendet VIC-Farbindizes 0..15
pub const PEPTO: [[u8; 3]; 16] = [[0x00, 0x00, 0x00], [0xff, 0xff, 0xff], [0x81, 0x33, 0x38], [0x75, 0xce, 0xc8], [0x8e, 0x3c, 0x97], [0x56, 0xac, 0x4d], [0x2e, 0x2c, 0x9b], [0xed, 0xf1, 0x71],
    [0x8e, 0x50, 0x29], [0x55, 0x38, 0x00], [0xc4, 0x6c, 0x71], [0x4a, 0x4a, 0x4a], [0x7b, 0x7b, 0x7b], [0xa9, 0xff, 0x9f], [0x70, 0x6d, 0xeb], [0xb2, 0xb2, 0xb2]];

/// Setzt die Pakete eines Bilds zu einem Bild aus Farbindizes zusammen. Fehlende Zeilen behalten den Inhalt des vorigen Bilds.
/// Die Höhe ergibt sich aus der größten gesehenen Zeile (PAL 272, NTSC 240).
#[derive(Debug, Clone)]
pub struct FrameAssembler {
    /// Farbindizes, `WIDTH` je Zeile, `MAX_LINES` Zeilen
    idx: Vec<u8>,
    height: u16,
    cur_frame: Option<u16>,
    complete: u64,
}

/// Ein fertiges Bild als RGBA
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame { pub width: usize, pub height: usize, pub rgba: Vec<u8>, pub frame_no: u16 }

impl Default for FrameAssembler { fn default() -> Self { FrameAssembler::new() } }

impl FrameAssembler {
    pub fn new() -> FrameAssembler { FrameAssembler { idx: vec![0; WIDTH as usize * MAX_LINES as usize], height: 0, cur_frame: None, complete: 0 } }
    /// Anzahl der bisher vollständig abgeschlossenen Bilder
    pub fn frames_done(&self) -> u64 { self.complete }
    pub fn height(&self) -> u16 { self.height }

    /// Rohes Paket einarbeiten. Liefert das Bild, wenn dieses Paket das letzte des Bilds war. Ungültige Pakete ergeben `Err` und ändern nichts.
    pub fn push(&mut self, raw: &[u8]) -> Result<Option<Frame>, super::PacketError> {
        let p = parse(raw)?;
        let h = p.header;
        let w = WIDTH as usize;
        for l in 0..h.lines_per_packet as usize {
            let row = &mut self.idx[(h.line as usize + l) * w..][..w];
            let src = &p.pixels[l * w / 2..][..w / 2];
            for (i, b) in src.iter().enumerate() { row[2 * i] = b & 0x0F; row[2 * i + 1] = b >> 4; }
        }
        self.cur_frame = Some(h.frame);
        self.height = self.height.max(h.line + h.lines_per_packet as u16);
        if h.last {
            self.height = h.line + h.lines_per_packet as u16;   // das letzte Paket legt die Höhe fest
            self.complete += 1;
            return Ok(Some(self.snapshot(h.frame)));
        }
        Ok(None)
    }

    /// Aktueller Stand als RGBA (Pepto)
    pub fn snapshot(&self, frame_no: u16) -> Frame {
        let (w, hh) = (WIDTH as usize, self.height as usize);
        let mut rgba = Vec::with_capacity(w * hh * 4);
        for &i in &self.idx[..w * hh] { let c = PEPTO[(i & 15) as usize]; rgba.extend_from_slice(&[c[0], c[1], c[2], 255]); }
        Frame { width: w, height: hh, rgba, frame_no }
    }
}

#[cfg(test)]
pub(crate) fn make(seq: u16, frame: u16, line: u16, last: bool) -> Vec<u8> {
    let mut p = vec![0u8; PACKET_LEN];
    p[0..2].copy_from_slice(&seq.to_le_bytes()); p[2..4].copy_from_slice(&frame.to_le_bytes());
    p[4..6].copy_from_slice(&(line | if last { 0x8000 } else { 0 }).to_le_bytes());
    p[6..8].copy_from_slice(&WIDTH.to_le_bytes()); p[8] = 4; p[9] = 4; p
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_header() {
        let mut p = make(7, 9, 268, true); p[HEADER_LEN] = 0x21;
        let v = parse(&p).unwrap();
        assert_eq!((v.header.seq, v.header.frame, v.header.line, v.header.last), (7, 9, 268, true));
        assert_eq!(v.pixels.len(), 768); assert_eq!(v.pixels[0], 0x21);
        assert!(!parse(&make(0, 0, 0, false)).unwrap().header.last);
    }
    #[test]
    fn rejects_bad_packets() {
        assert_eq!(parse(&[]).unwrap_err(), PacketError::Length { got: 0, want: 780 });
        assert!(matches!(parse(&make(0, 0, 0, false)[..779]), Err(PacketError::Length { .. })));
        let mut long = make(0, 0, 0, false); long.push(0); assert!(matches!(parse(&long), Err(PacketError::Length { .. })));
        for (i, v) in [(6usize, 0u8), (8, 2), (9, 8), (10, 1), (7, 2)] { let mut p = make(0, 0, 0, false); p[i] = v; assert!(matches!(parse(&p), Err(PacketError::Header(_))), "Byte {i}"); }
        assert!(matches!(parse(&make(0, 0, 300, false)), Err(PacketError::Header("Zeile außerhalb des Bilds"))));
        assert!(matches!(parse(&make(0, 0, 297, true)), Err(PacketError::Header(_))), "297+4 > 300");
        assert!(parse(&make(0, 0, 296, true)).is_ok());
    }

    #[test]
    fn assembler_builds_frames_and_keeps_old_lines() {
        let mut a = FrameAssembler::new();
        let mut p = make(0, 1, 0, false); p[HEADER_LEN] = 0x21;            // Pixel 0 = 1, Pixel 1 = 2
        assert!(a.push(&p).unwrap().is_none());
        let mut q = make(1, 1, 268, true); q[HEADER_LEN + 768 - 1] = 0xE6;  // letztes Pixel = 14, davor 6
        let f = a.push(&q).unwrap().expect("letztes Paket");
        assert_eq!((f.width, f.height, f.rgba.len()), (384, 272, 384 * 272 * 4));
        assert_eq!(&f.rgba[0..4], &[0xff, 0xff, 0xff, 255]);                // Index 1
        assert_eq!(&f.rgba[4..8], &[0x81, 0x33, 0x38, 255]);                // Index 2
        let last = (271 * 384 + 383) * 4; assert_eq!(&f.rgba[last..last + 3], &PEPTO[14]);
        assert_eq!(&f.rgba[last - 4..last - 1], &PEPTO[6]);
        // nächstes Bild ohne Zeile 0: alter Inhalt bleibt
        let f2 = a.push(&make(2, 2, 268, true)).unwrap().unwrap();
        assert_eq!(&f2.rgba[0..4], &[0xff, 0xff, 0xff, 255], "fehlende Zeilen behalten den alten Inhalt");
        assert_eq!(a.frames_done(), 2);
    }
    #[test]
    fn assembler_ntsc_height_and_errors() {
        let mut a = FrameAssembler::new();
        assert_eq!(a.push(&make(0, 0, 236, true)).unwrap().unwrap().height, 240);
        assert!(a.push(&[0u8; 100]).is_err());
        assert!(a.push(&make(0, 0, 400, false)).is_err());
        assert_eq!(a.height(), 240, "fehlerhafte Pakete ändern nichts");
    }
    #[test]
    fn pepto_matches_palette_file() {
        let t = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/palettes/c64.json")).unwrap();
        let hexes: Vec<&str> = t.split('"').filter(|x| x.len() == 7 && x.starts_with('#')).collect();
        assert_eq!(hexes.len(), 16);
        for (i, h) in hexes.iter().enumerate() {
            let rgb = [1, 3, 5].map(|k| u8::from_str_radix(&h[k..k + 2], 16).unwrap());
            assert_eq!(PEPTO[i], rgb, "Farbe {i}");
        }
    }
}
