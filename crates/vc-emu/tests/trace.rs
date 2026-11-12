// SPDX-License-Identifier: GPL-3.0-or-later
//! Auswertung des Debug-Stroms gegen berechnete Abläufe: ein kleines zyklengenaues 6502-Modell erzeugt die Buszyklen eines bekannten
//! Programms (mit Seitenübertrag, Verzweigungen, JSR/RTS, Interrupt, BA-Pausen), der Analysator muss Befehle, Zyklen und Abdeckung
//! daraus zurückgewinnen.
use vc_emu::streams::debug::{BusCycle, ENTRIES_PER_PACKET};
use vc_emu::streams::trace::*;
use vc_emu::streams::RxMeta;

/// (Opcode, Länge, Zyklen, Seitenübertrag) der im Modell benutzten Befehle
const OPS: &[(u8, u8, u8, bool)] = &[
    (0xA9, 2, 2, false), (0xA2, 2, 2, false), (0xA0, 2, 2, false), (0xAD, 3, 4, false), (0x8D, 3, 4, false), (0x9D, 3, 5, false), (0xBD, 3, 4, true),
    (0xE8, 1, 2, false), (0xCA, 1, 2, false), (0xD0, 2, 2, true), (0xF0, 2, 2, true), (0x4C, 3, 3, false), (0x20, 3, 6, false), (0x60, 1, 6, false),
    (0xEA, 1, 2, false), (0x40, 1, 6, false), (0xEE, 3, 6, false), (0xA5, 2, 3, false), (0x85, 2, 3, false), (0x6C, 3, 5, false), (0x58, 1, 2, false),
    (0x78, 1, 2, false), (0x48, 1, 3, false), (0x68, 1, 4, false), (0x00, 2, 7, false),
];

fn table() -> OpTable { OpTable::from_fn(|op| OPS.iter().find(|o| o.0 == op).map(|o| (o.1, o.2, o.3))) }

/// Wahrer Ablauf eines Befehls
#[derive(Debug, Clone, PartialEq)]
struct Truth { pc: u16, cycles: u8, start: u64, kind: InsnKind }

struct Sim { mem: Vec<u8>, pc: u16, a: u8, x: u8, y: u8, sp: u8, z: bool, trace: Vec<u32>, truth: Vec<Truth> }

impl Sim {
    fn new() -> Sim { Sim { mem: vec![0; 0x10000], pc: 0, a: 0, x: 0, y: 0, sp: 0xFF, z: false, trace: vec![], truth: vec![] } }
    fn load(&mut self, at: u16, bytes: &[u8]) { self.mem[at as usize..at as usize + bytes.len()].copy_from_slice(bytes); }
    fn rd(&mut self, a: u16) -> u8 { let v = self.mem[a as usize]; self.trace.push(BusCycle::new(a, v, true).0); v }
    fn wr(&mut self, a: u16, v: u8) { self.mem[a as usize] = v; self.trace.push(BusCycle::new(a, v, false).0); }
    fn fetch(&mut self) -> u8 { let v = self.rd(self.pc); self.pc = self.pc.wrapping_add(1); v }
    fn push(&mut self, v: u8) { self.wr(0x100 | self.sp as u16, v); self.sp = self.sp.wrapping_sub(1); }
    fn pull(&mut self) -> u8 { self.sp = self.sp.wrapping_add(1); self.rd(0x100 | self.sp as u16) }
    fn abs(&mut self) -> u16 { let lo = self.fetch(); let hi = self.fetch(); lo as u16 | (hi as u16) << 8 }

    fn step(&mut self) {
        let (pc0, t0) = (self.pc, self.trace.len());
        let op = self.fetch();
        match op {
            0xA9 => { self.a = self.fetch(); self.z = self.a == 0 }
            0xA2 => { self.x = self.fetch(); self.z = self.x == 0 }
            0xA0 => { self.y = self.fetch(); }
            0xAD => { let a = self.abs(); self.a = self.rd(a); self.z = self.a == 0 }
            0xA5 => { let a = self.fetch(); self.a = self.rd(a as u16); self.z = self.a == 0 }
            0x85 => { let a = self.fetch(); self.wr(a as u16, self.a) }
            0x8D => { let a = self.abs(); self.wr(a, self.a) }
            0x9D => { let a = self.abs(); let e = a.wrapping_add(self.x as u16); self.rd((a & 0xFF00) | (e & 0xFF)); self.wr(e, self.a) }
            0xBD => { let a = self.abs(); let e = a.wrapping_add(self.x as u16); if (a ^ e) & 0xFF00 != 0 { self.rd((a & 0xFF00) | (e & 0xFF)); } self.a = self.rd(e); self.z = self.a == 0 }
            0xE8 => { self.rd(self.pc); self.x = self.x.wrapping_add(1); self.z = self.x == 0 }
            0xCA => { self.rd(self.pc); self.x = self.x.wrapping_sub(1); self.z = self.x == 0 }
            0xEA | 0x58 | 0x78 => { self.rd(self.pc); }
            0x48 => { self.rd(self.pc); self.push(self.a) }
            0x68 => { self.rd(self.pc); self.rd(0x100 | self.sp as u16); self.a = self.pull(); }
            0xEE => { let a = self.abs(); let v = self.rd(a); self.wr(a, v); self.wr(a, v.wrapping_add(1)) }
            0xD0 | 0xF0 => {
                let rel = self.fetch() as i8; let take = if op == 0xD0 { !self.z } else { self.z };
                if take {
                    self.rd(self.pc);
                    let t = self.pc.wrapping_add(rel as u16);
                    if (t ^ self.pc) & 0xFF00 != 0 { self.rd((self.pc & 0xFF00) | (t & 0xFF)); }
                    self.pc = t;
                }
            }
            0x4C => { self.pc = self.abs() }
            0x6C => { let a = self.abs(); let lo = self.rd(a); let hi = self.rd((a & 0xFF00) | (a.wrapping_add(1) & 0xFF)); self.pc = lo as u16 | (hi as u16) << 8 }
            0x20 => {
                let lo = self.fetch(); self.rd(0x100 | self.sp as u16);
                let ret = self.pc; self.push((ret >> 8) as u8); self.push(ret as u8);
                let hi = self.fetch(); self.pc = lo as u16 | (hi as u16) << 8
            }
            0x60 => { self.rd(self.pc); self.rd(0x100 | self.sp as u16); let lo = self.pull(); let hi = self.pull(); self.rd(lo as u16 | (hi as u16) << 8); self.pc = (lo as u16 | (hi as u16) << 8).wrapping_add(1) }
            0x40 => { self.rd(self.pc); self.rd(0x100 | self.sp as u16); let _p = self.pull(); let lo = self.pull(); let hi = self.pull(); self.pc = lo as u16 | (hi as u16) << 8 }
            0x00 => { self.fetch(); self.interrupt_tail(0xFFFE) }
            o => panic!("Modell kennt {o:02X} nicht"),
        }
        let kind = InsnKind::Op;
        self.truth.push(Truth { pc: pc0, cycles: (self.trace.len() - t0) as u8, start: t0 as u64, kind });
    }

    fn interrupt_tail(&mut self, vec: u16) {
        let pc = self.pc; self.push((pc >> 8) as u8); self.push(pc as u8); self.push(0x20);
        let lo = self.rd(vec); let hi = self.rd(vec + 1); self.pc = lo as u16 | (hi as u16) << 8;
    }

    /// IRQ/NMI zwischen zwei Befehlen
    fn interrupt(&mut self, vec: u16) {
        let (pc0, t0) = (self.pc, self.trace.len());
        self.rd(self.pc); self.rd(self.pc); self.interrupt_tail(vec);
        self.truth.push(Truth { pc: pc0, cycles: 7, start: t0 as u64, kind: if vec == 0xFFFA { InsnKind::Nmi } else { InsnKind::Irq } });
    }
}

/// Bekanntes Programm: Schleife mit Seitenübertrag, Unterprogramm, Verzweigungen, SID-Schreibzugriff, Sprung durch Zeiger
fn program() -> Sim {
    let mut s = Sim::new();
    s.load(0x0810, &[
        0xA2, 0x05,             // 0810 LDX #5
        0xA9, 0x00,             // 0812 LDA #0
        0x20, 0x40, 0x08,       // 0814 JSR $0840
        0x9D, 0xFC, 0x08,       // 0817 STA $08FC,X     (Seitenübertrag je nach X: Schreiben hat immer 5 Zyklen)
        0xBD, 0xFC, 0x08,       // 081A LDA $08FC,X     (Übertrag ab X >= 4)
        0xCA,                   // 081D DEX
        0xD0, 0xF3,             // 081E BNE $0813?  (Ziel 0813 liegt mitten im Befehl: s.u. Korrektur)
    ]);
    // Zielkorrektur: Schleife zurück auf 0814
    s.mem[0x081F] = (0x0814i32 - 0x0820) as i8 as u8;
    s.load(0x0820, &[
        0x8D, 0x00, 0xD4,       // 0820 STA $D400
        0xEE, 0x05, 0xD4,       // 0823 INC $D405 (Lesen, Schreiben alt, Schreiben neu)
        0x6C, 0x30, 0x08,       // 0826 JMP ($0830)
    ]);
    s.load(0x0830, &[0x3A, 0x08]);                       // Zeiger auf $083A
    s.load(0x083A, &[0xEA, 0xF0, 0x02, 0xEA, 0xEA, 0x4C, 0x50, 0x08]);   // 083A NOP; BEQ +2 (genommen, wenn z); NOP; NOP; JMP $0850
    s.load(0x0840, &[0xE8, 0xCA, 0x60]);                 // 0840 INX; DEX; RTS
    s.load(0x0850, &[0x4C, 0x50, 0x08]);                 // 0850 JMP $0850 (Warteschleife)
    s.mem[0xFFFE] = 0x00; s.mem[0xFFFF] = 0x09; s.mem[0xFFFA] = 0x80; s.mem[0xFFFB] = 0x09;
    s.load(0x0900, &[0xE8, 0x40]);                       // IRQ-Behandlung: INX; RTI
    s.load(0x0980, &[0xEA, 0x40]);                       // NMI-Behandlung: NOP; RTI
    s.pc = 0x0810;
    s
}

fn packets(entries: &[u32]) -> Vec<Vec<u8>> {
    entries.chunks(ENTRIES_PER_PACKET).enumerate().map(|(i, ch)| {
        let mut p = vec![]; p.extend_from_slice(&(i as u16).to_le_bytes()); p.extend_from_slice(&[0, 0]);
        for e in ch { p.extend_from_slice(&e.to_le_bytes()); }
        p
    }).collect()
}

fn feed_all(an: &mut Analyzer, pk: &[Vec<u8>]) { for (i, p) in pk.iter().enumerate() { an.feed(p, &RxMeta { seq: i as u16, lost_before: 0, late: false }); } }

#[test]
fn recovers_instructions_cycles_and_coverage_of_a_known_program() {
    let mut s = program();
    for i in 0..400 {
        s.step();
        if i == 60 { s.interrupt(0xFFFE); s.step(); s.step(); }      // IRQ, dann INX und RTI der Behandlung
        if i == 120 { s.interrupt(0xFFFA); s.step(); s.step(); }
        if s.pc == 0x0850 && i > 200 { break; }
    }
    // Hinter der Schleife: Wartezeit im JMP *
    for _ in 0..300 { s.step(); }
    let pk = packets(&s.trace);
    let mut an = Analyzer::new(table());
    feed_all(&mut an, &pk);
    let sum = an.summary();
    assert_eq!(sum.cycles, s.trace.len() as u64);
    assert!(sum.synced, "{sum:?}");
    assert_eq!(sum.desyncs, 0, "{sum:?}");
    assert_eq!(sum.irqs, 1); assert_eq!(sum.nmis, 1);
    // Ohne bekannte Einstiege geht die Erkennung erst nach der Probe an: alles danach muss mit der Wahrheit übereinstimmen
    let rec = an.recent(1 << 20);
    assert!(rec.len() + 40 > s.truth.len(), "zu wenige Befehle erkannt: {} von {}", rec.len(), s.truth.len());
    let first = rec[0];
    let at = s.truth.iter().position(|t| t.start == first.start).expect("Beginn stimmt mit einem wahren Befehl überein");
    for (r, t) in rec.iter().zip(&s.truth[at..]) {
        assert_eq!((r.pc, r.cycles as u8, r.start, r.kind), (t.pc, t.cycles, t.start, t.kind), "Abweichung bei {t:?}");
    }
    assert_eq!(rec.len() + 1, s.truth.len() - at, "alle weiteren Befehle erkannt (der letzte ist noch offen)");
    // Abdeckung: genau die Adressen mit Opcode-Abruf (ab Einstieg)
    for t in &s.truth[at..] { if t.kind == InsnKind::Op { assert!(an.is_covered(t.pc), "{:04X}", t.pc); } }
    assert!(!an.is_covered(0x0825) && !an.is_covered(0x0819), "Operandbytes sind keine Befehlsstarts");
    assert!(!an.is_covered(0x083D), "nie ausgeführt (übersprungenes NOP)");
    // SID-Schreibzugriffe: $D400 einmal je Durchlauf, INC schreibt zweimal auf $D405
    let io = an.recent_io(1000);
    assert!(io.iter().any(|w| w.addr == 0xD400) && io.iter().filter(|w| w.addr == 0xD405).count() >= 2);
    assert!(io.windows(2).all(|w| w[0].cycle < w[1].cycle));
}

#[test]
fn known_entry_points_sync_immediately() {
    let mut s = program();
    for _ in 0..30 { s.step(); }
    let skip = 17;                                        // Strom beginnt mitten im Ablauf, am Beginn eines Befehls
    let from = s.truth[skip].start as usize;
    let pk = packets(&s.trace[from..]);
    let mut an = Analyzer::new(table());
    an.set_entry_points([0x0810u16, 0x0812, 0x0814, 0x0817, 0x081A, 0x081D, 0x081E, 0x0820, 0x0823, 0x0826, 0x083A, 0x083B, 0x0840, 0x0841, 0x0842, 0x0850]);
    feed_all(&mut an, &pk);
    let rec = an.recent(100);
    assert_eq!(rec.len(), s.truth.len() - skip - 1, "letzter Befehl ist noch nicht abgeschlossen");
    for (r, t) in rec.iter().zip(&s.truth[skip..]) { assert_eq!((r.pc, r.cycles as u8), (t.pc, t.cycles)); }
}

#[test]
fn ba_low_repeats_count_as_cycles_but_do_not_advance_instructions() {
    let mut s = program();
    for _ in 0..40 { s.step(); }
    // Vor dem Lesezyklus jedes LDA/STA/JSR (Datenlesen) 43 Wiederholungen mit BA = 0 einfügen
    let mut entries: Vec<u32> = vec![]; let mut stalls = 0u64;
    for (i, &e) in s.trace.iter().enumerate() {
        let b = BusCycle(e);
        if i % 50 == 25 && b.is_read() { for _ in 0..43 { entries.push(b.with_ba_low().0); stalls += 1; } }
        entries.push(e);
    }
    let mut an = Analyzer::new(table());
    an.set_entry_points([0x0810u16, 0x0812, 0x0814, 0x0817, 0x081A, 0x081D, 0x081E, 0x0820, 0x0823, 0x0826, 0x083A, 0x083B, 0x0840, 0x0841, 0x0842, 0x0850]);
    feed_all(&mut an, &packets(&entries));
    let sum = an.summary();
    assert_eq!(sum.cycles, entries.len() as u64); assert_eq!(sum.stall_cycles, stalls); assert_eq!(sum.desyncs, 0);
    let rec = an.recent(1000);
    let stall_in_trace: u64 = rec.iter().map(|r| r.stall as u64).sum();
    assert!(stall_in_trace > 0 && stall_in_trace <= stalls);
    for (r, t) in rec.iter().zip(s.truth.iter()) { assert_eq!((r.pc, r.cycles as u8), (t.pc, t.cycles)); }
}

#[test]
fn packet_loss_invalidates_running_measurement_and_restarts() {
    let mut s = program();
    for _ in 0..200 { s.step(); }
    // Messung: JSR-Abruf ($0814) bis zum nächsten Abruf hinter dem JSR ($0817)
    let pk = packets(&s.trace);
    let mut an = Analyzer::new(table());
    let id = an.add_measure(MeasureSpec { name: "Unterprogramm".into(), from: 0x0814, to: 0x0817 });
    feed_all(&mut an, &pk);
    let m = an.measure(id).unwrap();
    // JSR (6) + INX (2) + DEX (2) + RTS (6) = 16 Zyklen, nach der Wahrheit
    assert_eq!(m.min, Some(16)); assert_eq!(m.max, Some(16)); assert_eq!(m.invalid, 0);
    assert!(m.hint().contains("Hardware-Messung: 16 Zyklen") && m.hint().contains("kein Urteil"));

    // Mit Lücke: das Paket, in dem die Messung läuft, geht verloren -> ungültig, nie geschätzt
    let mut an = Analyzer::new(table());
    let id = an.add_measure(MeasureSpec { name: "x".into(), from: 0x0814, to: 0x0817 });
    // erstes Paket endet innerhalb der ersten Messung? Wir bauen es gezielt: Einträge bis mitten ins Unterprogramm, dann Lücke
    let start = s.truth.iter().find(|t| t.pc == 0x0814).unwrap().start as usize;
    let cut = start + 8;                                   // mitten im Unterprogramm
    let head = &s.trace[..cut]; let tail = &s.trace[cut + 3..];
    let mk = |seq: u16, e: &[u32]| { let mut p = seq.to_le_bytes().to_vec(); p.extend_from_slice(&[0, 0]); for x in e { p.extend_from_slice(&x.to_le_bytes()); } p };
    for (i, ch) in head.chunks(ENTRIES_PER_PACKET).enumerate() { an.feed(&mk(i as u16, ch), &RxMeta { seq: i as u16, lost_before: 0, late: false }); }
    let n = head.chunks(ENTRIES_PER_PACKET).count() as u16;
    let before = an.measure(id).unwrap();
    assert!(before.running, "die Messung läuft, als die Lücke kommt");
    an.feed(&mk(n + 1, &tail[..tail.len().min(ENTRIES_PER_PACKET)]), &RxMeta { seq: n + 1, lost_before: 1, late: false });
    let m = an.measure(id).unwrap();
    assert_eq!(m.invalid, 1, "{m:?}"); assert!(!m.running || m.completed > 0);
    let mut only_invalid = m.clone(); only_invalid.last = None; only_invalid.completed = 0;
    assert!(only_invalid.hint().contains("ungültig (Paketverlust)"));
    let sum = an.summary();
    assert_eq!(sum.lost_packets, 1); assert_eq!(sum.epochs, 1); assert_eq!(sum.epoch_cycles, tail.len().min(ENTRIES_PER_PACKET) as u64);
    // Die nächste Messung beginnt am nächsten Zugriff auf A: lauter gültige Werte, keiner aus der Lücke (sonst wäre ein Wert ≠ 16 dabei)
    let rest = &tail[tail.len().min(ENTRIES_PER_PACKET)..];
    let mut seq = n + 2;
    for ch in rest.chunks(ENTRIES_PER_PACKET) { an.feed(&mk(seq, ch), &RxMeta { seq, lost_before: 0, late: false }); seq += 1; }
    let m = an.measure(id).unwrap();
    assert_eq!(m.invalid, 1);
    assert!(m.completed >= 1 && m.values.iter().all(|&v| v == 16), "{m:?}");
    let starts = s.truth.iter().filter(|t| t.pc == 0x0814 && t.start as usize >= cut + 3).count() as u64;
    assert!(m.completed == starts || m.completed + 1 == starts, "{} von {starts}", m.completed);
}

#[test]
fn late_packets_are_ignored_and_foreign_modes_are_reported() {
    let mut an = Analyzer::new(table());
    let mut p = vec![0, 0, 0, 0]; p.extend_from_slice(&BusCycle::new(0x1000, 0xEA, true).0.to_le_bytes());
    an.feed(&p, &RxMeta { seq: 0, lost_before: 0, late: true });
    assert_eq!(an.summary().cycles, 0); assert_eq!(an.summary().late_packets, 1);
    let mut p = vec![0, 0, 0, 0]; p.extend_from_slice(&(BusCycle::new(0x1000, 0xEA, true).0 & 0x7FFF_FFFF).to_le_bytes());   // PHI2 = 0
    an.feed(&p, &RxMeta { seq: 0, lost_before: 0, late: false });
    let s = an.summary();
    assert!(s.unsupported.as_deref().unwrap_or("").contains("6510 Only")); assert_eq!(s.cycles, 0);
    // bleibt gesperrt
    let mut q = vec![1, 0, 0, 0]; q.extend_from_slice(&BusCycle::new(0x1000, 0xEA, true).0.to_le_bytes());
    an.feed(&q, &RxMeta { seq: 1, lost_before: 0, late: false });
    assert_eq!(an.summary().cycles, 0);
}

#[test]
fn garbage_does_not_panic_and_unknown_opcodes_desync() {
    let mut an = Analyzer::new(table());
    an.set_entry_points([0x1000u16]);
    let mut x = 0x1234_5678u32; let mut v = vec![];
    for _ in 0..5000 { x ^= x << 13; x ^= x >> 17; x ^= x << 5; v.push(x | 0x8000_0000); }
    feed_all(&mut an, &packets(&v));
    let s = an.summary(); assert_eq!(s.cycles, 5000);
    // zu kurze, zu lange und falsch ausgerichtete Pakete
    for n in [0usize, 1, 3, 4, 7, 1445, 1448] { an.feed(&vec![0u8; n], &RxMeta { seq: 0, lost_before: 0, late: false }); }
    assert_eq!(an.summary().cycles, 5000);
    // ein bekannter Einstieg mit unbekanntem Opcode ($02) ist ein Widerspruch
    let mut an = Analyzer::new(table()); an.set_entry_points([0x1000u16]);
    let v = [BusCycle::new(0x1000, 0x02, true).0, BusCycle::new(0x1001, 0xEA, true).0];
    feed_all(&mut an, &packets(&v));
    assert_eq!(an.summary().insns, 0);
}

#[test]
fn ring_holds_only_the_latest_instructions() {
    let mut s = program();
    for _ in 0..2000 { s.step(); }
    let mut an = Analyzer::with_ring_bytes(table(), 64 * std::mem::size_of::<InsnRec>());
    an.set_entry_points([0x0850u16]);
    feed_all(&mut an, &packets(&s.trace));
    let sum = an.summary();
    assert_eq!(sum.trace_len, 64); assert!(sum.trace_total > 64);
    let last = an.recent(5);
    assert!(last.windows(2).all(|w| w[0].start < w[1].start));
}

fn cyc(addr: u16, read: bool) -> u32 { addr as u32 | (0xFF00_0000 & !(if read { 0 } else { 1 << 24 })) }

#[test]
fn loader_writes_and_the_reset_vector_do_not_count_for_hardware_measurements() {
    // Das Laden über die Ultimate erscheint als Schreibzyklen an den Adressen des Programms; ein Reset als Lesen von $FFFC
    let run = |reads_only: bool, restart: bool| {
        let mut an = Analyzer::new(OpTable::empty());
        let id = an.add_measure(MeasureSpec { name: "m".into(), from: 0x0813, to: 0x081B });
        an.set_measure_reads_only(reads_only); an.set_restart_on_reset(restart);
        let mut t: Vec<u32> = vec![];
        t.extend([cyc(0x0813, true), cyc(0x0900, true), cyc(0x081B, true)]);                // altes Programm: 2 Zyklen
        t.extend([cyc(0xFFFC, true), cyc(0xFFFD, true)]);                                    // Reset
        t.extend([cyc(0x0813, false), cyc(0x0814, false), cyc(0x081B, false)]);            // Laden (Schreibzyklen)
        t.extend([cyc(0x0813, true), cyc(0x0900, true), cyc(0x0901, true), cyc(0x0902, true), cyc(0x081B, true)]);   // neues Programm: 4 Zyklen
        feed_all(&mut an, &packets(&t));
        (an.measure(id).unwrap(), an.summary())
    };
    let (m, s) = run(false, false);
    assert_eq!((s.resets, m.values.clone()), (1, vec![2, 2, 4]), "ohne Schutz zählt jedes: altes Programm, Laden, neues Programm");
    let (m, _) = run(true, false);
    assert_eq!(m.values, vec![2, 4], "ohne Schreibzugriffe: das Laden zählt nicht");
    let (m, s) = run(true, true);
    assert_eq!((m.values.clone(), m.completed, s.resets), (vec![4], 1, 1), "nach dem Reset von vorn: nur das neue Programm");
}
