// SPDX-License-Identifier: GPL-3.0-or-later
//! Dienst „Ströme“: startet und überwacht die UDP-Datenströme einer Ultimate (Bild, Ton, Ablaufprotokoll).
//!
//! Ablauf je Strom (Konzept „Strom-Steuerung per REST“): eigene Adresse bestimmen, Empfänger öffnen, `streams/<art>:start` aufrufen,
//! binnen 2 s auf das erste Paket warten, im Betrieb Rate und Verluste mitführen, bei Abbruch einmal neu starten, am Ende `:stop`.
//! Es werden keine Konfigurationswerte geschrieben und nichts im Flash gespeichert. Der Dienst startet nur auf ausdrücklichen Aufruf von
//! [`StreamService::start`], nie von selbst.
//!
//! Der Dienst hat keinen eigenen Thread: Die Oberfläche (oder `vcc`) ruft regelmäßig [`StreamService::poll`] auf. `start` und `stop`
//! machen REST-Aufrufe (bis zu einigen Sekunden bei unerreichbarem Gerät) und gehören daher nicht in den Zeichenthread.
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::{Duration, Instant};
use vc_emu::streams::{local_ip_towards, Delivery, ReceiverConfig, Sink, StatsSnapshot, StreamKind, StreamReceiver};
use vc_emu::Ultimate64;

/// Zustand eines Stroms (für den Statuschip)
#[derive(Debug, Clone, PartialEq)]
pub enum StreamState {
    Off,
    /// Gestartet, das erste Paket steht aus
    Starting,
    /// Pakete kommen an, Verluste unter 1 %
    Running,
    /// Pakete kommen an, ab 1 % Verlust (die Oberfläche färbt ab 5 % rot)
    Lossy,
    /// Nicht gestartet oder ausgefallen; Text für den Nutzer
    Error(String),
}

#[derive(Debug, Clone)]
pub struct StreamOptions {
    pub multicast: bool,
    /// Fester erster Port (Bild; Ton +1, Ablaufprotokoll +2), `None`/0 = automatisch
    pub base_port: Option<u16>,
    /// Eigene Adresse erzwingen
    pub interface: Option<Ipv4Addr>,
    /// Wartezeit auf das erste Paket und höchste Pause im Betrieb (Standard 2 s)
    pub timeout: Duration,
}

impl Default for StreamOptions { fn default() -> Self { StreamOptions { multicast: false, base_port: None, interface: None, timeout: Duration::from_secs(2) } } }

impl StreamOptions {
    pub fn from_settings(s: &crate::settings::Settings) -> StreamOptions {
        StreamOptions { multicast: s.streams.multicast, base_port: (s.streams.base_port != 0).then_some(s.streams.base_port), interface: s.streams.interface.trim().parse().ok(), ..Default::default() }
    }
}

/// Anzeigewerte eines Stroms
#[derive(Debug, Clone, PartialEq)]
pub struct StreamStatus {
    pub kind: StreamKind,
    pub state: StreamState,
    /// Ziel, das der Ultimate gemeldet wurde
    pub target: Option<SocketAddrV4>,
    pub packets: u64,
    pub lost: u64,
    pub rejected: u64,
    pub loss_percent: f64,
    /// Pakete je Sekunde und Mbit/s im letzten Messfenster
    pub packets_per_s: f64,
    pub mbit_per_s: f64,
    /// Alter des letzten Pakets
    pub since_last: Option<Duration>,
    /// Neustarts durch Abbruch seit dem Start
    pub restarts: u32,
}

struct Slot {
    state: StreamState,
    rx: Option<StreamReceiver>,
    sink: Option<Sink>,
    target: Option<SocketAddrV4>,
    opened: Instant,
    restarted: bool,
    restarts: u32,
    last_stats: Option<StatsSnapshot>,
    rate: (f64, f64),
    window: Option<(Instant, u64, u64)>,
}

impl Slot {
    fn new() -> Slot { Slot { state: StreamState::Off, rx: None, sink: None, target: None, opened: Instant::now(), restarted: false, restarts: 0, last_stats: None, rate: (0.0, 0.0), window: None } }
}

pub struct StreamService {
    host: String,
    password: Option<String>,
    opts: StreamOptions,
    dev: Option<Ultimate64>,
    slots: [Slot; 3],
}

fn idx(k: StreamKind) -> usize { match k { StreamKind::Video => 0, StreamKind::Audio => 1, StreamKind::Debug => 2 } }

/// Firewall-Hinweis (Punkt 1 des Hinweistextes). Unter Windows mit der Ergänzung zu Block-Regeln: ohne Antwort auf die Abfrage legt Windows
/// selbst eine Block-Regel für das Profil „Öffentlich“ an, „Privat zulassen“ hilft dann in einem als Öffentlich eingestuften Netz nicht.
pub fn firewall_advice(prog: &str, windows: bool) -> String {
    let base = format!("Firewall: eingehendes UDP für „{prog}“ erlauben (macOS: Systemeinstellungen → Netzwerk → Firewall; Windows: Defender-Firewall, Netzwerkprofil „Privat“). Die IDE ändert keine Firewall-Regel selbst.");
    if windows { format!("{base} Unter Windows: oder das Netz als Privat einstufen; bei einer Eingehend-Blockregel für {prog}.exe die Regel auf Zulassen ändern oder löschen und das Programm neu starten.") } else { base }
}

/// Hinweistext, wenn nach der Wartezeit kein Paket kam
pub fn no_packets_hint(kind: StreamKind, local: Option<SocketAddrV4>, multicast: bool) -> String {
    no_packets_hint_os(kind, local, multicast, cfg!(windows))
}

/// Wie [`no_packets_hint`] für ein Betriebssystem (testbar)
pub fn no_packets_hint_os(kind: StreamKind, local: Option<SocketAddrV4>, multicast: bool, windows: bool) -> String {
    let prog = std::env::current_exe().ok().and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned())).unwrap_or_else(|| "VintageCoder".into());
    let adr = local.map(|a| format!(" (Empfänger {a})")).unwrap_or_default();
    format!("Es kommen keine Pakete an (Strom „{}“{adr}). Bitte prüfen: \
1. {} \
2. Netz: Liegt die Ultimate im selben Netz wie diese Adresse? Bei mehreren Netzen die Schnittstelle in den Einstellungen festlegen. \
3. Ultimate: Hängt sie per LAN-Kabel am Netz (WLAN genügt nicht)? \
{}", kind.rest_name(), firewall_advice(&prog, windows), if multicast { "4. Multicast: Manche Switches und WLAN-Zugangspunkte leiten Multicast nicht weiter; im Zweifel Multicast in den Einstellungen abschalten." } else { "4. Hilft nichts: Multicast in den Einstellungen als Ausweg versuchen." })
}

impl StreamService {
    pub fn new(host: &str, password: Option<&str>, opts: StreamOptions) -> StreamService {
        StreamService { host: host.to_string(), password: password.filter(|p| !p.is_empty()).map(String::from), opts, dev: None, slots: [Slot::new(), Slot::new(), Slot::new()] }
    }
    /// Dienst für eine Maschine aus den Einstellungen
    pub fn for_machine(m: &crate::settings::Machine, s: &crate::settings::Settings) -> StreamService { StreamService::new(&m.host, Some(&m.password), StreamOptions::from_settings(s)) }
    pub fn host(&self) -> &str { &self.host }
    pub fn options(&self) -> &StreamOptions { &self.opts }
    pub fn state(&self, kind: StreamKind) -> &StreamState { &self.slots[idx(kind)].state }
    pub fn is_active(&self, kind: StreamKind) -> bool { self.slots[idx(kind)].rx.is_some() }

    /// Strom starten; die rohen Pakete gehen an `sink` (im Empfangsthread). Stoppt zuerst den Gegenstrom (Bild und Ablaufprotokoll nie zugleich)
    /// und einen schon laufenden Strom derselben Art. Kehrt nach dem REST-Aufruf zurück; das erste Paket meldet später [`poll`](Self::poll).
    pub fn start(&mut self, kind: StreamKind, sink: Sink) -> Result<(), String> {
        self.stop(kind);
        if let Some(c) = kind.counterpart() { if self.is_active(c) { self.stop(c); } }
        let s = &mut self.slots[idx(kind)];
        s.sink = Some(sink); s.restarted = false; s.restarts = 0;
        let r = self.open(kind);
        if let Err(e) = &r { self.fail(kind, e.clone()); }
        r
    }

    /// Strom beenden: Empfänger schließen und `:stop` senden (Fehler dabei werden ignoriert, z. B. wenn das Gerät aus ist)
    pub fn stop(&mut self, kind: StreamKind) {
        let had = self.slots[idx(kind)].rx.is_some();
        if had { if let Some(d) = &self.dev { let _ = d.stream_stop(kind); } }
        let s = &mut self.slots[idx(kind)];
        s.rx = None; s.sink = None; s.target = None; s.window = None; s.rate = (0.0, 0.0);
        s.state = StreamState::Off;
    }

    pub fn stop_all(&mut self) { for k in StreamKind::ALL { self.stop(k); } }

    fn fail(&mut self, kind: StreamKind, msg: String) {
        if self.slots[idx(kind)].rx.is_some() { if let Some(d) = &self.dev { let _ = d.stream_stop(kind); } }
        let s = &mut self.slots[idx(kind)];
        s.last_stats = s.rx.as_ref().map(|r| r.stats());
        s.rx = None; s.target = None; s.window = None; s.rate = (0.0, 0.0);
        s.state = StreamState::Error(msg);
    }

    /// Empfänger öffnen und `:start` senden (Schritte 1 bis 3)
    fn open(&mut self, kind: StreamKind) -> Result<(), String> {
        if self.dev.is_none() {
            let u = Ultimate64::connect(&self.host, self.password.as_deref()).map_err(|e| format!("Die Ultimate ist nicht erreichbar: {e}"))?;
            if let Ok(i) = u.info() { if !i.caps().streams { return Err(format!("Dieses Gerät ({}) sendet keine Datenströme; sie gibt es nur an der Ultimate 64.", i.product)); } }
            self.dev = Some(u);
        }
        let local = match self.opts.interface { Some(a) => a, None => local_ip_towards(&self.host)? };
        let (delivery, ip, port) = if self.opts.multicast {
            let (g, p) = kind.default_group();
            (Delivery::Multicast { group: g, port: p, iface: local }, g, p)
        } else {
            let port = self.opts.base_port.map(|b| b.saturating_add(idx(kind) as u16)).unwrap_or(0);
            (Delivery::Unicast { local, port }, local, port)
        };
        // Keine Filterung nach Absenderadresse: Die Ultimate sendet bei mehreren Schnittstellen (LAN und WLAN) mit wechselnder Absenderadresse.
        // Fremde Pakete scheitern ohnehin an der Prüfung von Länge und Kopf und werden gezählt.
        let source = None;
        let sink = self.slots[idx(kind)].sink.clone().ok_or("kein Empfänger für die Pakete angegeben")?;
        let rx = StreamReceiver::start(ReceiverConfig { kind, delivery, source, rcvbuf: 0 }, sink)?;
        let target = SocketAddrV4::new(ip, if port == 0 { rx.local_addr().port() } else { port });
        let started = self.dev.as_ref().ok_or("keine Verbindung")?.stream_start(kind, *target.ip(), target.port());
        started?;                                         // bei Fehler schließt sich `rx` beim Verlassen
        let s = &mut self.slots[idx(kind)];
        s.rx = Some(rx); s.target = Some(target); s.opened = Instant::now(); s.state = StreamState::Starting; s.window = None; s.last_stats = None;
        Ok(())
    }

    /// Zustände nachführen; regelmäßig aufrufen (etwa alle 100 bis 250 ms): erstes Paket, 2-s-Kontrolle, Neustart bei Abbruch, Raten.
    pub fn poll(&mut self) {
        for kind in StreamKind::ALL {
            let i = idx(kind);
            let Some(st) = self.slots[i].rx.as_ref().map(|r| r.stats()) else { continue };
            let now = Instant::now();
            let timeout = self.opts.timeout;
            let s = &mut self.slots[i];
            // Rate im Fenster von mindestens 0,5 s
            match s.window { None => s.window = Some((now, st.packets, st.bytes)), Some((t, p, b)) => { let dt = now.duration_since(t).as_secs_f64(); if dt >= 0.5 { s.rate = ((st.packets.saturating_sub(p)) as f64 / dt, st.bytes.saturating_sub(b) as f64 * 8.0 / dt / 1e6); s.window = Some((now, st.packets, st.bytes)); } } }
            s.last_stats = Some(st.clone());
            let stalled = match st.since_last { Some(d) => d > timeout, None => false };
            if st.packets == 0 {
                if now.duration_since(s.opened) > timeout {
                    let msg = if s.restarts > 0 { format!("Strom ausgefallen: auch nach einem Neustart kommen keine Pakete an. {}", no_packets_hint(kind, s.target, self.opts.multicast)) } else { no_packets_hint(kind, s.target, self.opts.multicast) };
                    self.fail(kind, msg);
                }
                continue;
            }
            if stalled {
                if !s.restarted {
                    s.restarted = true; s.restarts += 1;
                    s.rx = None;                                       // alten Empfänger schließen (mit :stop), dann Schritt 3 erneut
                    if let Some(d) = &self.dev { let _ = d.stream_stop(kind); }
                    if let Err(e) = self.open(kind) { self.fail(kind, format!("Strom ausgefallen, Neustart gescheitert: {e}")); }
                } else {
                    self.fail(kind, "Strom ausgefallen: Seit mehr als 2 Sekunden kommt kein Paket mehr an (Gerät aus, Kabel gezogen oder Strom in der Ultimate beendet).".into());
                }
                continue;
            }
            s.restarted = false;
            s.state = if st.loss_percent() >= 1.0 { StreamState::Lossy } else { StreamState::Running };
        }
    }

    /// Anzeigewerte; ohne laufenden Strom bleiben die Zähler des letzten Laufs stehen
    pub fn status(&self, kind: StreamKind) -> StreamStatus {
        let s = &self.slots[idx(kind)];
        let st = s.rx.as_ref().map(|r| r.stats()).or_else(|| s.last_stats.clone());
        let (packets, lost, rejected, loss, since) = st.map(|x| (x.packets, x.lost, x.rejected, x.loss_percent(), x.since_last)).unwrap_or((0, 0, 0, 0.0, None));
        StreamStatus { kind, state: s.state.clone(), target: s.target, packets, lost, rejected, loss_percent: loss, packets_per_s: s.rate.0, mbit_per_s: s.rate.1, since_last: since, restarts: s.restarts }
    }
}

impl Drop for StreamService { fn drop(&mut self) { self.stop_all(); } }
