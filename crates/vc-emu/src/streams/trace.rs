// SPDX-License-Identifier: GPL-3.0-or-later
//! Auswertung des Debug-Stroms (Modus „6510 Only“) ohne Stau.
//!
//! Der [`Analyzer`] arbeitet im Empfangsthread: Er liest ein Paket, zählt Zyklen weiter, findet Befehlsgrenzen und aktualisiert Zähler,
//! Abdeckung, Ablaufprotokoll (Ringpuffer) und Messungen, bevor das nächste Paket kommt. Einzelne Ereignisse gehen nie an andere Threads;
//! Leser holen sich Schnappschüsse ([`Summary`], [`Analyzer::recent`], [`Analyzer::coverage`]).
//!
//! # Befehlsgrenzen
//! Der Strom hat kein SYNC-Bit. Ein Befehl beginnt am Zyklus, der nach Ablauf der Mindestzyklen der Opcode-Tabelle ([`OpTable`]) von der
//! erwarteten Folgeadresse liest (`PC + Länge`; bei JMP/JSR/RTS/RTI/BRK/Interrupt aus den gelesenen Daten, bei Verzweigungen aus dem
//! Operanden). Dazwischen liegende Zyklen (Operanden, Datenzugriffe, Seitenübertrag) gehören zum Befehl. Interrupt-Einsprünge erkennt man
//! am zweiten Zyklus: Er liest dieselbe Adresse wie der erste. Passt die Folge nicht mehr (unbekannter Opcode, zu viele Zyklen), gilt der
//! Strom als „nicht synchron“ ([`Summary::desyncs`]); wieder eingestiegen wird an einer bekannten Befehlsadresse (falls bekannt) oder
//! nach einer Probe, die mindestens [`MIN_SYNC_INSNS`] Befehle widerspruchsfrei durchläuft.
//!
//! # Zyklen und BA
//! Jeder Eintrag ist ein Zyklus; die Zyklenzähler stützen sich nie auf die Befehlsgrenzen. Hält der VIC den Bus (BA niedrig), wird ein
//! Lesezyklus der CPU wiederholt: Diese Wiederholungen zählen als Zyklen (sie kosten Zeit) und zugleich in [`Summary::stall_cycles`],
//! führen die Befehlserkennung aber nicht weiter.
//!
//! # Verlust
//! Fehlt eine Paketnummer, beginnt eine neue *Epoche*: Zyklenzähler, Befehlserkennung und laufende Messungen werden verworfen
//! (Messung „ungültig (Paketverlust)“), nichts wird geschätzt.
//!
//! Nur der NMOS-6502/6510 wird unterstützt (keine 45GS02). Enthält ein Paket Einträge mit PHI2 = 0, stammt es aus einem Modus mit VIC- oder
//! 1541-Zyklen: Der Analysator meldet das in [`Summary::unsupported`] und wertet nichts aus.
use super::debug::{self, BusCycle};
use super::{RxMeta, Sink};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};

/// Größe des Ablaufprotokolls (Ringpuffer) in Byte, wenn nichts anderes verlangt wird (rund eine Sekunde)
pub const DEFAULT_RING_BYTES: usize = 4 << 20;
/// So viele Befehle müssen nach einem Einstieg ohne Widerspruch durchlaufen, ehe der Strom als synchron gilt
pub const MIN_SYNC_INSNS: usize = 8;
/// So viele Einzelwerte hebt eine Messung auf
pub const MEASURE_KEEP: usize = 1000;
/// Größe des Ringpuffers der SID/VIC-Schreibzugriffe (Einträge)
pub const IO_RING: usize = 1 << 16;

/// Art des Befehls für die Folgeadresse
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpKind { Normal, Branch, Jmp, JmpInd, Jsr, Rts, Rti, Brk }

/// Eintrag der Opcode-Tabelle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpInfo {
    /// Länge in Byte (Opcode und Operanden)
    pub len: u8,
    /// Zyklen ohne Zuschläge
    pub cycles: u8,
    /// Ein Zyklus mehr bei Seitenübertrag
    pub page_cross: bool,
    pub kind: OpKind,
}

/// Opcode-Tabelle des NMOS-6502/6510. Die Längen und Zyklen kommen aus der CPU-Beschreibung (`vc-target`), diese Struktur hängt nicht davon ab.
#[derive(Clone)]
pub struct OpTable { ops: Box<[Option<OpInfo>; 256]> }

impl OpTable {
    pub fn empty() -> OpTable { OpTable { ops: Box::new([None; 256]) } }
    /// Tabelle aus einer Funktion `Opcode → (Länge, Zyklen, Seitenübertrag)`. Die Befehlsart (JMP, JSR, RTS, RTI, BRK, Verzweigung)
    /// folgt aus dem Opcode des NMOS-6502.
    pub fn from_fn(f: impl Fn(u8) -> Option<(u8, u8, bool)>) -> OpTable {
        let mut t = OpTable::empty();
        for op in 0..=255u8 {
            let Some((len, cycles, page_cross)) = f(op) else { continue };
            let kind = match op { 0x00 => OpKind::Brk, 0x20 => OpKind::Jsr, 0x40 => OpKind::Rti, 0x60 => OpKind::Rts, 0x4C => OpKind::Jmp, 0x6C => OpKind::JmpInd, o if o & 0x1F == 0x10 => OpKind::Branch, _ => OpKind::Normal };
            if len == 0 || len > 3 || cycles == 0 || cycles > 8 { continue; }
            t.ops[op as usize] = Some(OpInfo { len, cycles, page_cross, kind });
        }
        t
    }
    pub fn get(&self, opcode: u8) -> Option<OpInfo> { self.ops[opcode as usize] }
    pub fn len(&self) -> usize { self.ops.iter().flatten().count() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

/// Art eines Ablaufeintrags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsnKind { Op, Irq, Nmi }

/// Ein Eintrag des Ablaufprotokolls: ein ausgeführter Befehl (oder Interrupt-Einsprung)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsnRec {
    /// Epoche (zählt bei jedem Paketverlust weiter); Startzeiten sind nur innerhalb einer Epoche vergleichbar
    pub epoch: u32,
    /// Zyklus seit Beginn der Epoche, an dem der Befehl begann
    pub start: u64,
    /// Adresse des Opcode-Abrufs (bei einem Interrupt: der Befehl, der als Nächstes gelaufen wäre)
    pub pc: u16,
    /// Zyklen des Befehls ohne die Wiederholungen durch BA
    pub cycles: u8,
    /// Wiederholte Lesezyklen durch BA = 0 innerhalb des Befehls
    pub stall: u16,
    pub kind: InsnKind,
}

/// Schreibzugriff auf SID ($D400–$D41C) oder VIC ($D000–$D02E) mit Zeitstempel in Zyklen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoWrite { pub epoch: u32, pub cycle: u64, pub addr: u16, pub value: u8 }

pub fn is_io_watch(addr: u16) -> bool { (0xD400..=0xD41C).contains(&addr) || (0xD000..=0xD02E).contains(&addr) }

/// Ringpuffer fester Größe (der älteste Eintrag fällt heraus)
#[derive(Debug, Clone)]
pub struct Ring<T: Copy> { buf: Vec<T>, cap: usize, next: usize, pushed: u64 }

impl<T: Copy> Ring<T> {
    pub fn new(cap: usize) -> Ring<T> { Ring { buf: Vec::new(), cap: cap.max(1), next: 0, pushed: 0 } }
    pub fn push(&mut self, v: T) {
        if self.buf.len() < self.cap { self.buf.push(v); } else { self.buf[self.next] = v; }
        self.next = (self.next + 1) % self.cap; self.pushed += 1;
    }
    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn capacity(&self) -> usize { self.cap }
    /// Insgesamt eingefügte Einträge (auch die schon herausgefallenen)
    pub fn total(&self) -> u64 { self.pushed }
    /// Die letzten `n` Einträge, ältester zuerst
    pub fn last(&self, n: usize) -> Vec<T> {
        let n = n.min(self.buf.len());
        if self.buf.len() < self.cap { return self.buf[self.buf.len() - n..].to_vec(); }
        (0..n).map(|i| self.buf[(self.next + self.cap - n + i) % self.cap]).collect()
    }
    pub fn clear(&mut self) { self.buf.clear(); self.next = 0; }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// Befehlserkennung

#[derive(Debug, Clone, Copy)]
struct Cyc { addr: u16, data: u8, read: bool, idx: u64 }

#[derive(Debug, Clone)]
struct Cur {
    pc: u16, op: OpInfo, start: u64, n: u8, stall: u16,
    /// Daten und Adressen der Zyklen (Index = Zyklusnummer − 1)
    data: [u8; 10],
    /// Unterbrechung (IRQ/NMI/BRK-Folge): 7 Zyklen, Folgeadresse aus dem Vektor
    irq: bool,
    /// Adresse des 6. Zyklus (Vektorabruf: $FFFA = NMI, $FFFE = IRQ)
    a6: u16,
    /// Mindest- und Höchstzahl der Zyklen
    min: u8, max: u8,
    /// Folgeadresse einer genommenen Verzweigung
    taken: Option<u16>,
    /// Verzweigung: Zyklus 3 gelesen, ob genommen oder nicht, entscheidet der nächste Zyklus
    pending: Option<Cyc>,
}

#[derive(Debug, Clone)]
enum Ev { Insn(InsnRec), Desync }

#[derive(Clone)]
struct Machine { table: Arc<OpTable>, cur: Option<Cur>, epoch: u32 }

fn le(lo: u8, hi: u8) -> u16 { lo as u16 | (hi as u16) << 8 }

impl Machine {
    fn new(table: Arc<OpTable>, epoch: u32) -> Machine { Machine { table, cur: None, epoch } }

    fn expected(cur: &Cur) -> Option<u16> {
        if cur.irq { return Some(le(cur.data[5], cur.data[6])); }
        Some(match cur.op.kind {
            OpKind::Normal => cur.pc.wrapping_add(cur.op.len as u16),
            OpKind::Branch => return cur.taken,
            OpKind::Jmp => le(cur.data[1], cur.data[2]),
            OpKind::Jsr => le(cur.data[1], cur.data[5]),
            OpKind::JmpInd => le(cur.data[3], cur.data[4]),
            OpKind::Rts => le(cur.data[3], cur.data[4]).wrapping_add(1),
            OpKind::Rti => le(cur.data[4], cur.data[5]),
            OpKind::Brk => le(cur.data[5], cur.data[6]),
        })
    }

    fn finish(&mut self, out: &mut Vec<Ev>) {
        let Some(cur) = self.cur.take() else { return };
        let kind = if cur.irq { if cur.a6 == 0xFFFA { InsnKind::Nmi } else { InsnKind::Irq } } else { InsnKind::Op };
        out.push(Ev::Insn(InsnRec { epoch: self.epoch, start: cur.start, pc: cur.pc, cycles: cur.n, stall: cur.stall, kind }));
    }

    /// Beginn eines Befehls am Zyklus `c`; `false`, wenn er keiner sein kann
    fn begin(&mut self, c: Cyc) -> bool {
        if !c.read { return false; }
        let Some(op) = self.table.get(c.data) else { return false };
        let mut data = [0u8; 10]; data[0] = c.data;
        self.cur = Some(Cur { pc: c.addr, op, start: c.idx, n: 1, stall: 0, data, irq: false, a6: 0, min: op.cycles, max: op.cycles + (op.page_cross as u8), taken: None, pending: None });
        true
    }

    /// Ein Zyklus. `ba_high`: CPU hat den Bus. Gibt `false` zurück bei Widerspruch (dann ist `cur` leer und ein `Desync` gemeldet).
    fn push(&mut self, c: Cyc, ba_high: bool, out: &mut Vec<Ev>) -> bool {
        if c.read && !ba_high { if let Some(cur) = self.cur.as_mut() { cur.stall = cur.stall.saturating_add(1); } return true; }
        if self.cur.is_none() {
            if self.begin(c) { return true; }
            out.push(Ev::Desync); return false;
        }
        // Verzweigung: Entscheidung über Zyklus 3 liegt vor?
        let cur = self.cur.as_mut().unwrap();
        if let Some(p) = cur.pending.take() {
            let pc2 = cur.pc.wrapping_add(2);
            let t = cur.taken.unwrap_or(pc2);
            if c.read && c.addr == pc2.wrapping_add(1) && t != pc2.wrapping_add(1) {
                // nicht genommen: Zyklus 3 war schon der Abruf des nächsten Befehls
                cur.taken = None;
                self.finish(out);
                if !self.begin(p) { out.push(Ev::Desync); return false; }
                return self.push(c, true, out);
            }
            let cross = (pc2 & 0xFF00) != (t & 0xFF00);
            cur.n = 3; cur.data[2] = p.data; cur.min = 3; cur.max = 3 + cross as u8;
        }
        let cur = self.cur.as_mut().unwrap();
        // Unterbrechung: der zweite Zyklus liest dieselbe Adresse wie der erste
        if cur.n == 1 && c.read && c.addr == cur.pc && !cur.irq {
            cur.irq = true; cur.min = 7; cur.max = 7;
            cur.data[1] = c.data; cur.n = 2;
            return true;
        }
        if cur.n >= cur.min && c.read {
            if cur.op.kind == OpKind::Branch && !cur.irq && cur.n == 2 && cur.taken.is_none() {
                let pc2 = cur.pc.wrapping_add(2);
                if c.addr == pc2 {
                    cur.taken = Some(pc2.wrapping_add(cur.data[1] as i8 as u16));
                    cur.pending = Some(c);
                    return true;
                }
            } else if Machine::expected(cur) == Some(c.addr) {
                self.finish(out);
                self.begin(c);
                if self.cur.is_none() { out.push(Ev::Desync); return false; }
                return true;
            }
        }
        if cur.n >= cur.max { self.cur = None; out.push(Ev::Desync); return false; }
        let i = cur.n as usize;
        if i < cur.data.len() { cur.data[i] = c.data; }
        if i == 5 { cur.a6 = c.addr; }
        cur.n += 1;
        true
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// Messungen

/// Gemessen wird die Zahl der Zyklen vom ersten Zugriff auf `from` bis zum ersten danach auf `to`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureSpec { pub name: String, pub from: u16, pub to: u16 }

/// Stand einer Messung. Jede Messung beginnt neu am nächsten Zugriff auf `from`; läuft über eine Lücke in den Paketnummern eine Messung, zählt sie
/// als ungültig und liefert keinen Wert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureResult {
    pub spec: MeasureSpec,
    /// Fertige, gültige Messungen
    pub completed: u64,
    /// Durch Paketverlust ungültige Messungen
    pub invalid: u64,
    pub last: Option<u64>,
    pub min: Option<u64>,
    pub max: Option<u64>,
    /// Die ersten [`MEASURE_KEEP`] Werte
    pub values: Vec<u64>,
    /// Es läuft gerade eine Messung (A gesehen, B steht aus)
    pub running: bool,
}

impl MeasureResult {
    /// Text für Hardware-Läufe: der Wert ist eine Messung, kein Urteil
    pub fn hint(&self) -> String {
        match (self.last, self.invalid) {
            (Some(v), 0) => format!("Hardware-Messung: {v} Zyklen von ${:04X} bis ${:04X} (Hinweis, kein Urteil)", self.spec.from, self.spec.to),
            (Some(v), n) => format!("Hardware-Messung: {v} Zyklen von ${:04X} bis ${:04X} (Hinweis, kein Urteil; {n} Messung(en) ungültig wegen Paketverlust)", self.spec.from, self.spec.to),
            (None, 0) => format!("Hardware-Messung: ${:04X} bis ${:04X} noch nicht durchlaufen", self.spec.from, self.spec.to),
            (None, n) => format!("Hardware-Messung: ungültig (Paketverlust), {n} Messung(en) verworfen"),
        }
    }
}

#[derive(Debug, Clone)]
struct Measure { spec: MeasureSpec, running: Option<u64>, completed: u64, invalid: u64, last: Option<u64>, min: Option<u64>, max: Option<u64>, values: Vec<u64> }

// ---------------------------------------------------------------------------------------------------------------------------------
// Analysator

/// Zählerstand zu einem Zeitpunkt
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Summary {
    pub packets: u64,
    /// Nach Paketnummern fehlende Pakete
    pub lost_packets: u64,
    /// Zu spät oder doppelt eingetroffene Pakete (ignoriert)
    pub late_packets: u64,
    /// Neuanfänge durch Paketverlust
    pub epochs: u32,
    /// Alle Zyklen seit dem Start (über Epochen)
    pub cycles: u64,
    /// Zyklen der laufenden Epoche
    pub epoch_cycles: u64,
    /// Wiederholte Lesezyklen mit BA = 0 (in `cycles` enthalten)
    pub stall_cycles: u64,
    pub insns: u64,
    pub irqs: u64,
    pub nmis: u64,
    /// Lesezugriffe auf den Reset-Vektor `$FFFC` (die CPU holt ihn nach einem Reset; Programme lesen ihn sonst fast nie)
    pub resets: u64,
    /// Widersprüche der Befehlserkennung (unbekannter Opcode, Folge passt nicht)
    pub desyncs: u64,
    /// Wie oft die Befehlserkennung wieder eingestiegen ist
    pub syncs: u64,
    pub synced: bool,
    /// Adressen mit Opcode-Abruf
    pub covered: usize,
    /// Einträge des Ablaufprotokolls im Ring (und seit Beginn insgesamt)
    pub trace_len: usize,
    pub trace_total: u64,
    /// SID/VIC-Schreibzugriffe insgesamt
    pub io_writes: u64,
    /// Von der Aufzeichnung (`set_recorder`) nicht übernommene Befehle, weil der Schreiber nicht mitkam
    pub recorder_dropped: u64,
    /// Grund, warum nicht ausgewertet wird (Modus mit VIC/1541)
    pub unsupported: Option<String>,
}

impl Summary {
    pub fn loss_percent(&self) -> f64 { let t = self.packets + self.lost_packets; if t == 0 { 0.0 } else { 100.0 * self.lost_packets as f64 / t as f64 } }
}

pub struct Analyzer {
    table: Arc<OpTable>,
    m: Machine,
    synced: bool,
    trial: Vec<(Cyc, bool)>,
    starts: Option<Box<[u64; 1024]>>,
    sum: Summary,
    ecyc: u64,
    prev_stalled: bool,
    prev_addr: u16,
    cov: Box<[u64; 1024]>,
    ring: Ring<InsnRec>,
    io: Ring<IoWrite>,
    measures: Vec<Measure>,
    restart_on_reset: bool,
    reads_only: bool,
    recorder: Option<SyncSender<Vec<InsnRec>>>,
    batch: Vec<InsnRec>,
    evs: Vec<Ev>,
}

fn bit_set(map: &mut [u64; 1024], a: u16) { map[(a >> 6) as usize] |= 1 << (a & 63); }
fn bit_get(map: &[u64; 1024], a: u16) -> bool { map[(a >> 6) as usize] >> (a & 63) & 1 != 0 }

impl Analyzer {
    pub fn new(table: OpTable) -> Analyzer { Analyzer::with_ring_bytes(table, DEFAULT_RING_BYTES) }
    pub fn with_ring_bytes(table: OpTable, ring_bytes: usize) -> Analyzer {
        let table = Arc::new(table);
        Analyzer { m: Machine::new(table.clone(), 0), table, synced: false, trial: vec![], starts: None, sum: Summary::default(), ecyc: 0, prev_stalled: false, prev_addr: 0, cov: Box::new([0; 1024]),
            ring: Ring::new((ring_bytes / std::mem::size_of::<InsnRec>()).max(16)), io: Ring::new(IO_RING), measures: vec![], restart_on_reset: false, reads_only: false, recorder: None, batch: vec![], evs: vec![] }
    }
    /// Bekannte Befehlsadressen (aus der `.vcdbg`): an einer solchen Adresse steigt die Erkennung sofort ein, statt eine Probe zu durchlaufen
    pub fn set_entry_points(&mut self, addrs: impl IntoIterator<Item = u16>) {
        let mut m = Box::new([0u64; 1024]); let mut any = false;
        for a in addrs { bit_set(&mut m, a); any = true; }
        self.starts = any.then_some(m);
    }
    /// Alle Befehle zusätzlich an einen Schreiber geben (Pakete von Befehlen, nicht blockierend: ist der Kanal voll, werden Befehle verworfen und gezählt)
    pub fn set_recorder(&mut self, tx: Option<SyncSender<Vec<InsnRec>>>) { self.recorder = tx; }
    pub fn add_measure(&mut self, spec: MeasureSpec) -> usize {
        self.measures.push(Measure { spec, running: None, completed: 0, invalid: 0, last: None, min: None, max: None, values: vec![] });
        self.measures.len() - 1
    }
    /// Nach einem Reset (Lesezugriff auf `$FFFC`) alle Messungen von vorn beginnen: so zählen Zugriffe des vorigen Programms nicht,
    /// wenn die Hardware zurückgesetzt und ein Programm geladen wird, während der Strom schon läuft.
    pub fn set_restart_on_reset(&mut self, on: bool) { self.restart_on_reset = on; }
    /// Nur Lesezugriffe (Abruf, Lesen) zählen als Zugriff auf `from` und `to`; Schreibzugriffe werden übergangen. Beim Laden eines Programms
    /// über die Ultimate erscheint jedes geladene Byte als Schreibzyklus an seiner Adresse und würde sonst die Messung starten.
    pub fn set_measure_reads_only(&mut self, on: bool) { self.reads_only = on; }
    /// Alle Messungen von vorn beginnen (laufende verwerfen, Werte löschen), z. B. nachdem das Programm neu gestartet wurde: Zugriffe des
    /// vorigen Programms zählen nicht. Zyklenzähler, Epochen und Abdeckung bleiben.
    pub fn reset_measures(&mut self) {
        for m in &mut self.measures { *m = Measure { spec: m.spec.clone(), running: None, completed: 0, invalid: 0, last: None, min: None, max: None, values: vec![] }; }
    }
    pub fn measure(&self, id: usize) -> Option<MeasureResult> {
        self.measures.get(id).map(|m| MeasureResult { spec: m.spec.clone(), completed: m.completed, invalid: m.invalid, last: m.last, min: m.min, max: m.max, values: m.values.clone(), running: m.running.is_some() })
    }
    pub fn summary(&self) -> Summary {
        let mut s = self.sum.clone();
        s.epoch_cycles = self.ecyc; s.synced = self.synced; s.trace_len = self.ring.len(); s.trace_total = self.ring.total();
        s.covered = self.cov.iter().map(|w| w.count_ones() as usize).sum();
        s
    }
    /// Die letzten `n` Befehle des Ablaufprotokolls, ältester zuerst
    pub fn recent(&self, n: usize) -> Vec<InsnRec> { self.ring.last(n) }
    pub fn recent_io(&self, n: usize) -> Vec<IoWrite> { self.io.last(n) }
    pub fn is_covered(&self, addr: u16) -> bool { bit_get(&self.cov, addr) }
    /// Bitkarte der Abdeckung (64 KB = 1024 Wörter zu 64 Adressen; Adresse `a` ist Bit `a & 63` von Wort `a >> 6`)
    pub fn coverage(&self) -> Vec<u64> { self.cov.to_vec() }
    pub fn clear_coverage(&mut self) { self.cov.fill(0); }
    pub fn ring_capacity(&self) -> usize { self.ring.capacity() }

    /// Neuanfang nach Paketverlust: nichts wird geschätzt
    fn restart_epoch(&mut self) {
        self.sum.epochs += 1; self.ecyc = 0; self.prev_stalled = false;
        self.m = Machine::new(self.table.clone(), self.sum.epochs);
        self.synced = false; self.trial.clear();
        for m in &mut self.measures { if m.running.take().is_some() { m.invalid += 1; } }
    }

    /// Ein Paket aus dem Empfangsthread
    pub fn feed(&mut self, data: &[u8], meta: &RxMeta) {
        let Ok(p) = debug::parse(data) else { return };
        if meta.late { self.sum.late_packets += 1; return; }
        if meta.lost_before > 0 { self.sum.lost_packets += meta.lost_before as u64; if self.sum.packets > 0 { self.restart_epoch(); } }
        self.sum.packets += 1;
        if self.sum.unsupported.is_some() { return; }
        if p.entries().any(|e| !BusCycle(e).phi2()) {
            self.sum.unsupported = Some("Der Debug-Strom enthält Zyklen mit PHI2 = 0: Er läuft nicht im Modus „6510 Only“. Modi mit VIC- oder 1541-Zyklen werden nicht ausgewertet; bitte in der Ultimate unter Data Streams → Debug Stream Mode „6510 Only“ wählen.".into());
            return;
        }
        for e in p.entries() { self.cycle(BusCycle(e)); }
        if let Some(tx) = &self.recorder {
            if !self.batch.is_empty() {
                let n = self.batch.len() as u64;
                if tx.try_send(std::mem::take(&mut self.batch)).is_err() { self.sum.recorder_dropped += n; self.batch.clear(); }
            }
        }
    }

    fn cycle(&mut self, b: BusCycle) {
        let idx = self.ecyc; self.ecyc += 1; self.sum.cycles += 1;
        let c = Cyc { addr: b.addr(), data: b.data(), read: b.is_read(), idx };
        let stalled = c.read && !b.ba_high();
        if stalled { self.sum.stall_cycles += 1; }
        // Messungen: Zugriffe, wobei die Wiederholungen eines durch BA angehaltenen Lesezyklus (und sein Abschluss) derselbe Zugriff bleiben
        let same_access = self.prev_stalled && c.read && self.prev_addr == c.addr;
        self.prev_stalled = stalled; self.prev_addr = c.addr;
        if c.read && c.addr == 0xFFFC && !same_access { self.sum.resets += 1; if self.restart_on_reset { self.reset_measures(); } }
        let skip = same_access || (self.reads_only && !c.read);
        for m in self.measures.iter_mut().filter(|_| !skip) {
            if let Some(start) = m.running {
                if c.addr == m.spec.to {
                    let v = idx - start; m.running = None; m.completed += 1; m.last = Some(v);
                    m.min = Some(m.min.map_or(v, |x| x.min(v))); m.max = Some(m.max.map_or(v, |x| x.max(v)));
                    if m.values.len() < MEASURE_KEEP { m.values.push(v); }
                }
            }
            if m.running.is_none() && c.addr == m.spec.from { m.running = Some(idx); }
        }
        if !c.read && is_io_watch(c.addr) { self.sum.io_writes += 1; self.io.push(IoWrite { epoch: self.sum.epochs, cycle: idx, addr: c.addr, value: c.data }); }
        if self.synced { self.step_live(c, b.ba_high()); } else { self.step_unsynced(c, b.ba_high()); }
    }

    fn step_live(&mut self, c: Cyc, ba: bool) {
        let mut evs = std::mem::take(&mut self.evs);
        let ok = self.m.push(c, ba, &mut evs);
        self.apply(&mut evs);
        self.evs = evs;
        if !ok { self.sum.desyncs += 1; self.synced = false; self.trial.clear(); self.m.cur = None; if !(c.read && !ba) { self.trial.push((c, ba)); } }
    }

    fn apply(&mut self, evs: &mut Vec<Ev>) {
        for ev in evs.drain(..) {
            if let Ev::Insn(r) = ev {
                self.sum.insns += 1;
                match r.kind { InsnKind::Irq => self.sum.irqs += 1, InsnKind::Nmi => self.sum.nmis += 1, InsnKind::Op => bit_set(&mut self.cov, r.pc) }
                self.ring.push(r);
                if self.recorder.is_some() { self.batch.push(r); }
            }
        }
    }

    fn step_unsynced(&mut self, c: Cyc, ba: bool) {
        if let Some(st) = &self.starts {
            if c.read && ba && bit_get(st, c.addr) && self.m.begin(c) {
                self.trial.clear(); self.synced = true; self.sum.syncs += 1; return;
            }
        }
        self.trial.push((c, ba));
        if self.trial.len() >= 96 && self.trial.len() % 16 == 0 { self.try_sync(); }
        if !self.synced && self.trial.len() > 384 { self.trial.drain(..192); }
    }

    fn try_sync(&mut self) {
        for k in 0..self.trial.len().min(48) {
            let mut m = Machine::new(self.table.clone(), self.sum.epochs);
            let mut evs = vec![]; let mut ok = true;
            for &(c, ba) in &self.trial[k..] { if !m.push(c, ba, &mut evs) { ok = false; break; } }
            if ok && evs.iter().filter(|e| matches!(e, Ev::Insn(_))).count() >= MIN_SYNC_INSNS {
                self.m = m; self.synced = true; self.sum.syncs += 1; self.trial.clear();
                self.apply(&mut evs);
                return;
            }
        }
    }
}

/// Analysator, wie ihn Empfangsthread und Oberfläche gemeinsam benutzen
pub type SharedAnalyzer = Arc<Mutex<Analyzer>>;

pub fn shared(a: Analyzer) -> SharedAnalyzer { Arc::new(Mutex::new(a)) }

/// Sink, die jedes Paket sofort im Empfangsthread auswertet (kurze Sperre, kein Kanal, kein Stau)
pub fn analyzer_sink(a: &SharedAnalyzer) -> Sink {
    let a = a.clone();
    Arc::new(move |d: &[u8], m: RxMeta| { if let Ok(mut g) = a.lock() { g.feed(d, &m); } })
}

/// Zyklen zwischen dem ersten Zugriff auf `from` und dem ersten danach auf `to` in einer Folge von Einträgen (ohne Paketverlust)
pub fn measure_entries(entries: impl IntoIterator<Item = u32>, from: u16, to: u16) -> Option<u64> {
    let mut start = None;
    for (i, e) in entries.into_iter().enumerate() {
        let a = BusCycle(e).addr();
        match start { Some(s) if a == to => return Some(i as u64 - s), None if a == from => start = Some(i as u64), _ => {} }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ring_keeps_last_entries() {
        let mut r = Ring::new(4);
        for i in 0..3 { r.push(i); }
        assert_eq!(r.last(10), vec![0, 1, 2]);
        for i in 3..10 { r.push(i); }
        assert_eq!(r.last(3), vec![7, 8, 9]); assert_eq!(r.last(10), vec![6, 7, 8, 9]); assert_eq!(r.total(), 10);
    }
    #[test]
    fn measure_entries_counts_between_first_accesses() {
        let v: Vec<u32> = [0x10u16, 0x20, 0x30, 0x20, 0x40, 0x30].iter().map(|&a| BusCycle::new(a, 0, true).0).collect();
        assert_eq!(measure_entries(v.iter().copied(), 0x20, 0x40), Some(3));
        assert_eq!(measure_entries(v.iter().copied(), 0x40, 0x10), None);
        assert_eq!(measure_entries(v.iter().copied(), 0x30, 0x30), Some(3), "A = B misst den Abstand zweier Zugriffe");
    }
}
