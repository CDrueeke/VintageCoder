// SPDX-License-Identifier: GPL-3.0-or-later
//! UDP-Empfänger für einen Strom: ein Socket, ein Thread, großer Empfangspuffer, Zähler. Reicht geprüfte rohe Pakete an eine [`Sink`].
use super::{PacketError, SeqEvent, SeqTracker, StreamKind};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Angaben zum Paket, die der Empfänger zusätzlich zu den rohen Bytes liefert
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxMeta {
    pub seq: u16,
    /// Pakete, die unmittelbar vor diesem fehlen (0 = lückenlos)
    pub lost_before: u16,
    /// Paket kam zu spät oder doppelt (Reihenfolge verletzt)
    pub late: bool,
}

/// Empfänger der rohen Pakete. Läuft im Empfangsthread: kurz halten, nicht blockieren.
pub type Sink = Arc<dyn Fn(&[u8], RxMeta) + Send + Sync>;

/// Paket samt Angaben, wie es [`channel_sink`] weiterreicht
#[derive(Debug, Clone)]
pub struct RawPacket { pub data: Vec<u8>, pub meta: RxMeta }

/// Sink, die Pakete in einen begrenzten Kanal legt (`cap` Pakete). Ist er voll, wird das Paket verworfen (der Empfangsthread wartet nie).
pub fn channel_sink(cap: usize) -> (Sink, mpsc::Receiver<RawPacket>) {
    let (tx, rx) = mpsc::sync_channel(cap.max(1));
    (Arc::new(move |d: &[u8], m: RxMeta| { let _ = tx.try_send(RawPacket { data: d.to_vec(), meta: m }); }), rx)
}

/// Wie die Pakete zum Mac kommen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Gezielt an die eigene Adresse; `port` 0 = freien Port wählen lassen
    Unicast { local: Ipv4Addr, port: u16 },
    /// Gruppe auf der Schnittstelle mit der Adresse `iface` abonnieren
    Multicast { group: Ipv4Addr, port: u16, iface: Ipv4Addr },
}

#[derive(Debug, Clone, Copy)]
pub struct ReceiverConfig {
    pub kind: StreamKind,
    pub delivery: Delivery,
    /// Nur Pakete von dieser Adresse annehmen (die Ultimate); `None` = jede
    pub source: Option<Ipv4Addr>,
    /// Gewünschter Empfangspuffer in Byte (0 = 4 MB)
    pub rcvbuf: usize,
}

#[derive(Debug, Default)]
struct Counters { packets: AtomicU64, bytes: AtomicU64, lost: AtomicU64, late: AtomicU64, rejected: AtomicU64, control: AtomicU64, foreign: AtomicU64, last_ns: AtomicU64 }

/// Zählerstand zu einem Zeitpunkt
#[derive(Debug, Clone, PartialEq)]
pub struct StatsSnapshot {
    /// Angenommene Nutzpakete
    pub packets: u64,
    pub bytes: u64,
    /// Nach Folgenummern fehlende Pakete (nachgelieferte werden wieder abgezogen)
    pub lost: u64,
    /// Zu spät oder doppelt eingetroffen
    pub late: u64,
    /// Verworfen wegen Länge oder Kopf
    pub rejected: u64,
    /// Steuerpakete (2 Byte beim Start)
    pub control: u64,
    /// Von einer anderen als der erwarteten Quelle
    pub foreign: u64,
    /// Zeit seit dem letzten Nutz- oder Steuerpaket; `None`, wenn noch keines kam
    pub since_last: Option<Duration>,
    /// Zeit seit dem Start des Empfängers
    pub elapsed: Duration,
}

impl StatsSnapshot {
    /// Anteil fehlender Pakete in Prozent
    pub fn loss_percent(&self) -> f64 { let t = self.packets + self.lost; if t == 0 { 0.0 } else { 100.0 * self.lost as f64 / t as f64 } }
}

pub struct StreamReceiver {
    kind: StreamKind,
    local: SocketAddrV4,
    rcvbuf: usize,
    started: Instant,
    counters: Arc<Counters>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for StreamReceiver { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("StreamReceiver").field("kind", &self.kind).field("local", &self.local).finish() } }

/// Die lokale Adresse, über die das Betriebssystem die Ultimate erreicht: UDP-Socket „verbinden“ (es wird nichts gesendet), lokale Adresse lesen.
/// `host` ist `adresse` oder `adresse:port` (Port irrelevant).
pub fn local_ip_towards(host: &str) -> Result<Ipv4Addr, String> {
    let target = if host.contains(':') { host.to_string() } else { format!("{host}:80") };
    let sa = target.to_socket_addrs().map_err(|e| format!("{host}: {e}"))?.find(|a| a.is_ipv4()).ok_or_else(|| format!("{host}: keine IPv4-Adresse"))?;
    let s = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("UDP-Socket: {e}"))?;
    s.connect(sa).map_err(|_| "Der Rechner erreicht die Ultimate nicht (keine Route).".to_string())?;
    match s.local_addr().map_err(|e| e.to_string())? {
        SocketAddr::V4(a) if !a.ip().is_unspecified() => Ok(*a.ip()),
        _ => Err("Der Rechner erreicht die Ultimate nicht (keine Route).".into()),
    }
}

fn bind_err(e: std::io::Error, addr: SocketAddrV4) -> String {
    match e.kind() {
        std::io::ErrorKind::AddrInUse => format!("Port {} ist belegt.", addr.port()),
        std::io::ErrorKind::PermissionDenied => format!("Keine Berechtigung, UDP-Port {} zu öffnen (Rechte oder Firewall).", addr.port()),
        _ => format!("UDP-Empfänger {addr}: {e}"),
    }
}

impl StreamReceiver {
    /// Socket öffnen und den Empfangsthread starten
    pub fn start(cfg: ReceiverConfig, sink: Sink) -> Result<StreamReceiver, String> {
        let (bind, group) = match cfg.delivery {
            Delivery::Unicast { local, port } => (SocketAddrV4::new(local, port), None),
            Delivery::Multicast { group, port, iface } => (SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port), Some((group, iface))),
        };
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).map_err(|e| format!("UDP-Socket: {e}"))?;
        if group.is_some() {   // mehrere Mitleser auf demselben Gruppenport
            let _ = sock.set_reuse_address(true);
            #[cfg(unix)] let _ = sock.set_reuse_port(true);
        }
        let want = if cfg.rcvbuf == 0 { 4 << 20 } else { cfg.rcvbuf };
        let _ = sock.set_recv_buffer_size(want);                       // Das System darf kürzen; wir arbeiten auch mit weniger
        let rcvbuf = sock.recv_buffer_size().unwrap_or(0);
        sock.bind(&SocketAddr::V4(bind).into()).map_err(|e| bind_err(e, bind))?;
        let sock: UdpSocket = sock.into();
        if let Some((g, iface)) = group { sock.join_multicast_v4(&g, &iface).map_err(|e| format!("Multicast-Gruppe {g} auf {iface}: {e}"))?; }
        sock.set_read_timeout(Some(Duration::from_millis(100))).map_err(|e| e.to_string())?;
        let local = match sock.local_addr().map_err(|e| e.to_string())? { SocketAddr::V4(a) => a, _ => return Err("keine IPv4-Adresse".into()) };
        let counters = Arc::new(Counters::default()); let stop = Arc::new(AtomicBool::new(false));
        let started = Instant::now();
        let (c, s, kind, source) = (counters.clone(), stop.clone(), cfg.kind, cfg.source);
        let thread = std::thread::Builder::new().name(format!("vc-stream-{}", kind.rest_name()))
            .spawn(move || run(sock, kind, source, started, c, s, sink)).map_err(|e| format!("Empfangsthread: {e}"))?;
        Ok(StreamReceiver { kind, local, rcvbuf, started, counters, stop, thread: Some(thread) })
    }
    pub fn kind(&self) -> StreamKind { self.kind }
    /// Adresse und Port, an die die Ultimate senden soll (bei Unicast mit freiem Port: der gewählte Port)
    pub fn local_addr(&self) -> SocketAddrV4 { self.local }
    /// Tatsächlicher Empfangspuffer des Sockets in Byte (0 = unbekannt)
    pub fn recv_buffer(&self) -> usize { self.rcvbuf }
    pub fn stats(&self) -> StatsSnapshot {
        let c = &self.counters; let now = self.started.elapsed();
        let last = c.last_ns.load(Relaxed);
        StatsSnapshot {
            packets: c.packets.load(Relaxed), bytes: c.bytes.load(Relaxed), lost: c.lost.load(Relaxed), late: c.late.load(Relaxed), rejected: c.rejected.load(Relaxed),
            control: c.control.load(Relaxed), foreign: c.foreign.load(Relaxed),
            since_last: if last == 0 { None } else { Some(now.saturating_sub(Duration::from_nanos(last))) }, elapsed: now,
        }
    }
    /// Thread beenden und Socket schließen (auch bei `drop`)
    pub fn stop(mut self) { self.shutdown(); }
    fn shutdown(&mut self) { self.stop.store(true, Relaxed); if let Some(t) = self.thread.take() { let _ = t.join(); } }
}

impl Drop for StreamReceiver { fn drop(&mut self) { self.shutdown(); } }

fn run(sock: UdpSocket, kind: StreamKind, source: Option<Ipv4Addr>, started: Instant, c: Arc<Counters>, stop: Arc<AtomicBool>, sink: Sink) {
    let mut buf = vec![0u8; 4096];                  // größer als das größte Paket (1444), damit zu lange Pakete nicht abgeschnitten gültig aussehen
    let mut seq = SeqTracker::new();
    let stamp = || (started.elapsed().as_nanos() as u64).max(1);
    while !stop.load(Relaxed) {
        let (n, from) = match sock.recv_from(&mut buf) {
            Ok(x) => x,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted) => continue,
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => continue,
            Err(_) => { std::thread::sleep(Duration::from_millis(50)); continue }
        };
        if let (Some(src), SocketAddr::V4(f)) = (source, from) { if *f.ip() != src { c.foreign.fetch_add(1, Relaxed); continue; } }
        let p = &buf[..n];
        let s = match kind.check(p) {
            Ok(s) => s,
            Err(PacketError::Control) => { c.control.fetch_add(1, Relaxed); c.last_ns.store(stamp(), Relaxed); continue }
            Err(_) => { c.rejected.fetch_add(1, Relaxed); continue }
        };
        let (lost_before, late) = match seq.observe(s) {
            SeqEvent::First | SeqEvent::InOrder => (0, false),
            SeqEvent::Gap(g) => { c.lost.fetch_add(g as u64, Relaxed); (g, false) }
            SeqEvent::Late => { c.late.fetch_add(1, Relaxed); let v = c.lost.load(Relaxed); c.lost.store(v.saturating_sub(1), Relaxed); (0, true) }
        };
        c.packets.fetch_add(1, Relaxed); c.bytes.fetch_add(n as u64, Relaxed); c.last_ns.store(stamp(), Relaxed);
        sink(p, RxMeta { seq: s, lost_before, late });
    }
}
