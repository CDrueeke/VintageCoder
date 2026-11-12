// SPDX-License-Identifier: GPL-3.0-or-later
//! Datenströme der Ultimate 64 (UDP): Video, Audio und Debug (Buszyklen).
//!
//! Dieses Modul liefert den Empfänger ([`StreamReceiver`]: Socket, Thread, Zähler), die reinen Kopf-Decoder je Strom
//! ([`video`], [`audio`], [`debug`]) und die Folgenummernprüfung. Die Auswertung der Nutzdaten (Pixel, Samples, Zyklen)
//! gehört den Verbrauchern, die rohe Pakete über eine [`Sink`] erhalten.
//!
//! Alle Pakete sind nicht vertrauenswürdig: Decoder prüfen jede Länge und jeden Wert und liefern [`PacketError`] statt zu
//! verfallen; verworfene Pakete werden gezählt.
use std::fmt;

pub mod audio;
pub mod debug;
mod receiver;
pub mod trace;
pub mod video;
pub mod wav;

pub use receiver::{channel_sink, local_ip_towards, Delivery, RxMeta, Sink, StatsSnapshot, StreamReceiver, ReceiverConfig, RawPacket};

/// Art des Stroms (Name im REST-Pfad `/v1/streams/<name>:start`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StreamKind { Video, Audio, Debug }

impl StreamKind {
    pub const ALL: [StreamKind; 3] = [StreamKind::Video, StreamKind::Audio, StreamKind::Debug];
    /// Name im REST-Pfad
    pub fn rest_name(self) -> &'static str { match self { StreamKind::Video => "video", StreamKind::Audio => "audio", StreamKind::Debug => "debug" } }
    /// Deutscher Anzeigename
    pub fn label(self) -> &'static str { match self { StreamKind::Video => "Bild", StreamKind::Audio => "Ton", StreamKind::Debug => "Ablaufprotokoll" } }
    /// Standard-Multicast-Gruppe und -Port der Ultimate (Menü „Data Streams“)
    pub fn default_group(self) -> (std::net::Ipv4Addr, u16) {
        match self { StreamKind::Video => ([239, 0, 1, 64].into(), 11000), StreamKind::Audio => ([239, 0, 1, 65].into(), 11001), StreamKind::Debug => ([239, 0, 1, 66].into(), 11002) }
    }
    /// Der Strom, der nie zugleich laufen darf (Video und Debug teilen sich die Bandbreite der Ultimate)
    pub fn counterpart(self) -> Option<StreamKind> { match self { StreamKind::Video => Some(StreamKind::Debug), StreamKind::Debug => Some(StreamKind::Video), StreamKind::Audio => None } }
    /// Paket prüfen und die Folgenummer liefern. `Err(PacketError::Control)` für Steuerpakete (Audio-Start), die kein Nutzpaket sind.
    pub fn check(self, p: &[u8]) -> Result<u16, PacketError> {
        if p.len() == 2 { return Err(PacketError::Control); }   // jeder Strom beginnt mit einem Paket von 2 Byte (an der Hardware gemessen)
        match self {
            StreamKind::Video => video::parse(p).map(|v| v.header.seq),
            StreamKind::Audio => audio::parse(p).map(|a| a.seq),
            StreamKind::Debug => debug::parse(p).map(|d| d.seq),
        }
    }
}

/// Warum ein Paket verworfen wurde
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    /// Falsche Länge
    Length { got: usize, want: usize },
    /// Kopf unplausibel (Beschreibung des Feldes)
    Header(&'static str),
    /// Steuerpaket ohne Nutzdaten (2 Byte beim Start jedes Stroms); kein Fehler, wird nicht weitergereicht
    Control,
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketError::Length { got, want } => write!(f, "Paketlänge {got} statt {want}"),
            PacketError::Header(w) => write!(f, "Paketkopf unplausibel: {w}"),
            PacketError::Control => write!(f, "Steuerpaket"),
        }
    }
}
impl std::error::Error for PacketError {}

/// Ergebnis der Folgenummernprüfung für ein Paket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqEvent {
    /// Erstes Paket des Stroms
    First,
    /// Erwartete Nummer
    InOrder,
    /// So viele Pakete fehlen davor
    Gap(u16),
    /// Spät eingetroffen oder doppelt (Nummer liegt hinter der erwarteten)
    Late,
}

/// Verfolgt die laufende 16-Bit-Folgenummer (mit Überlauf)
#[derive(Debug, Clone, Default)]
pub struct SeqTracker { next: Option<u16> }

impl SeqTracker {
    pub fn new() -> SeqTracker { SeqTracker { next: None } }
    pub fn observe(&mut self, seq: u16) -> SeqEvent {
        let Some(exp) = self.next else { self.next = Some(seq.wrapping_add(1)); return SeqEvent::First };
        let d = seq.wrapping_sub(exp);
        if d == 0 { self.next = Some(seq.wrapping_add(1)); SeqEvent::InOrder }
        else if d < 0x8000 { self.next = Some(seq.wrapping_add(1)); SeqEvent::Gap(d) }
        else { SeqEvent::Late }
    }
    /// Neu aufsetzen (nach Neustart des Stroms)
    pub fn reset(&mut self) { self.next = None; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seq_tracker_counts_gaps_wrap_and_late() {
        let mut t = SeqTracker::new();
        assert_eq!(t.observe(10), SeqEvent::First);
        assert_eq!(t.observe(11), SeqEvent::InOrder);
        assert_eq!(t.observe(14), SeqEvent::Gap(2));
        assert_eq!(t.observe(13), SeqEvent::Late);
        assert_eq!(t.observe(15), SeqEvent::InOrder);
        let mut t = SeqTracker::new(); t.observe(0xFFFE);
        assert_eq!(t.observe(0xFFFF), SeqEvent::InOrder); assert_eq!(t.observe(0), SeqEvent::InOrder);
        assert_eq!(t.observe(2), SeqEvent::Gap(1));
        assert_eq!(t.observe(2), SeqEvent::Late, "doppelt");
    }
    #[test]
    fn kinds_have_rest_names_and_counterparts() {
        assert_eq!(StreamKind::Video.rest_name(), "video");
        assert_eq!(StreamKind::Video.counterpart(), Some(StreamKind::Debug));
        assert_eq!(StreamKind::Audio.counterpart(), None);
        assert_eq!(StreamKind::Debug.default_group(), ([239, 0, 1, 66].into(), 11002));
    }
}
