// SPDX-License-Identifier: GPL-3.0-or-later
//! Debug-Strom (Modus „6510 Only“): bis 1444 Byte je Paket = ein 32-Bit-Wort Kopf (16 Bit Zähler, 16 Bit reserviert) und bis 360 Einträge zu 32 Bit
//! (little-endian), ein Eintrag je CPU-Zyklus. Meist sind es volle Pakete; an der Hardware gemessen kommen daneben regelmäßig kürzere (etwa 868 Byte
//! alle 55 Pakete, das erste Paket nach dem Start 608 Byte) mit lückenloser Folgenummer. Gültig ist jede Länge `4 + 4·n` mit `1 ≤ n ≤ 360`. Die Bits eines Eintrags entschlüsselt [`BusCycle`]; die Auswertung (Befehlsgrenzen, Zyklen, Abdeckung) steht in [`super::trace`].
use super::PacketError;

pub const PACKET_LEN: usize = 1444;
pub const ENTRIES_PER_PACKET: usize = 360;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugPacket<'a> { pub seq: u16, pub reserved: u16, /// bis 1440 Byte = bis 360 Einträge
    pub data: &'a [u8] }

impl DebugPacket<'_> {
    /// Einträge als 32-Bit-Wörter
    pub fn entries(&self) -> impl Iterator<Item = u32> + '_ { self.data.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])) }
}

/// Ein Eintrag = ein CPU-Zyklus im Modus „6510 Only“.
///
/// Bits 15..0 Adresse, 23..16 Daten, 24 R/W# (1 = Lesen), 25 NMI#, 31 PHI2. **Abweichung vom Konzept, an der Hardware gemessen**
/// (Ultimate 64 II, Firmware 3.14d, `tests/data/debug.bin`): Bit 27 verhält sich wie **BA** (niedrig genau in den 43 Zyklen jeder Bad Line,
/// alle 504 Zyklen) und Bit 26 wie **IRQ#** (niedrig vom Auslösen des Raster-IRQs bis zum Quittieren in `$D019`). Bit 28 war in allen
/// Aufnahmen 1, auch bei Lesezugriffen aufs KERNAL-ROM; seine Bedeutung ist ungeklärt und wird nicht ausgewertet.
/// Bit 29/30 sind EXROM#/GAME# (immer 1 in den Aufnahmen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BusCycle(pub u32);

impl BusCycle {
    pub fn addr(self) -> u16 { self.0 as u16 }
    pub fn data(self) -> u8 { (self.0 >> 16) as u8 }
    pub fn is_read(self) -> bool { self.0 & (1 << 24) != 0 }
    pub fn nmi_asserted(self) -> bool { self.0 & (1 << 25) == 0 }
    pub fn irq_asserted(self) -> bool { self.0 & (1 << 26) == 0 }
    /// `true`: die CPU hat den Bus; `false` (BA niedrig): der VIC holt sich den Bus, Lesezyklen der CPU werden angehalten und wiederholt
    pub fn ba_high(self) -> bool { self.0 & (1 << 27) != 0 }
    pub fn exrom_low(self) -> bool { self.0 & (1 << 29) == 0 }
    pub fn game_low(self) -> bool { self.0 & (1 << 30) == 0 }
    /// PHI2 = 1 in allen Einträgen des Modus „6510 Only“; ein Eintrag mit PHI2 = 0 stammt aus einem Modus mit VIC- oder 1541-Zyklen
    pub fn phi2(self) -> bool { self.0 & (1 << 31) != 0 }
    /// Eintrag aus Adresse, Daten und Lese-Flag (alle Zustandsbits 1, PHI2 gesetzt): für Tests und Aufzeichnungen
    pub fn new(addr: u16, data: u8, read: bool) -> BusCycle { BusCycle(0xFE00_0000 | (read as u32) << 24 | (data as u32) << 16 | addr as u32) }
    /// Wie `new`, aber mit BA niedrig (Bus beim VIC)
    pub fn with_ba_low(self) -> BusCycle { BusCycle(self.0 & !(1 << 27)) }
}

pub fn parse(p: &[u8]) -> Result<DebugPacket<'_>, PacketError> {
    if p.len() < 8 || p.len() > PACKET_LEN || p.len() % 4 != 0 { return Err(PacketError::Length { got: p.len(), want: PACKET_LEN }); }
    Ok(DebugPacket { seq: u16::from_le_bytes([p[0], p[1]]), reserved: u16::from_le_bytes([p[2], p[3]]), data: &p[4..] })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_entries() {
        let mut p = vec![0u8; PACKET_LEN]; p[0] = 0x34; p[1] = 0x12; p[4..8].copy_from_slice(&0x8001_00FFu32.to_le_bytes());
        let d = parse(&p).unwrap(); assert_eq!(d.seq, 0x1234); assert_eq!(d.entries().count(), ENTRIES_PER_PACKET);
        assert_eq!(d.entries().next(), Some(0x8001_00FF));
    }
    #[test]
    fn accepts_short_packets_and_rejects_bad_lengths() {
        for n in [8usize, 608, 868, 1444] { let v = vec![0; n]; let d = parse(&v).unwrap(); assert_eq!(d.entries().count(), (n - 4) / 4); }
        for n in [0usize, 1, 2, 3, 4, 5, 7, 1441, 1443, 1445, 1448, 2000] { assert!(matches!(parse(&vec![0; n]), Err(PacketError::Length { .. })), "{n}"); }
    }
}
