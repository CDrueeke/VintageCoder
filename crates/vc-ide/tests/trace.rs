// SPDX-License-Identifier: GPL-3.0-or-later
//! Ablaufprotokoll gegen echte Pakete (`vc-emu/tests/data/debug.bin`, Ultimate 64 II, „6510 Only“) mit der Opcode-Tabelle des 6510.
//! Hardware: `U64_HOST=… cargo test -p vc-ide --test trace -- --ignored --nocapture` (die macOS-Firewall muss das Testprogramm für UDP
//! zulassen; sonst `vcc streams trace`).
use std::path::PathBuf;
use vc_emu::streams::trace::{Analyzer, InsnKind, MeasureSpec};
use vc_emu::streams::RxMeta;
use vc_ide::trace::{format_record, op_table, LineMap};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
fn packets() -> Vec<Vec<u8>> {
    let b = std::fs::read(root().join("crates/vc-emu/tests/data/debug.bin")).unwrap();
    let (mut o, mut out) = (0, vec![]);
    while o + 2 <= b.len() { let n = u16::from_le_bytes([b[o], b[o + 1]]) as usize; out.push(b[o + 2..o + 2 + n].to_vec()); o += 2 + n; }
    out
}
fn analyzer() -> Analyzer { Analyzer::new(op_table(&vc_target::load_cpu(&root().join("data"), "6510").unwrap())) }
fn feed(an: &mut Analyzer, ps: &[Vec<u8>]) { for p in ps { let seq = u16::from_le_bytes([p[0], p[1]]); an.feed(p, &RxMeta { seq, lost_before: 0, late: false }); } }

#[test]
fn op_table_of_the_6510_has_the_documented_shape() {
    let t = op_table(&vc_target::load_cpu(&root().join("data"), "6510").unwrap());
    let lda_abs = t.get(0xAD).unwrap(); assert_eq!((lda_abs.len, lda_abs.cycles), (3, 4));
    let abx = t.get(0xBD).unwrap(); assert!(abx.page_cross && abx.cycles == 4);
    let sta_abx = t.get(0x9D).unwrap(); assert!(!sta_abx.page_cross && sta_abx.cycles == 5);
    let bne = t.get(0xD0).unwrap(); assert_eq!((bne.len, bne.cycles), (2, 2));
    assert_eq!((t.get(0x20).unwrap().cycles, t.get(0x60).unwrap().cycles, t.get(0x40).unwrap().cycles, t.get(0x00).unwrap().cycles), (6, 6, 6, 7));
    assert!(t.get(0x02).is_none(), "JAM ist kein Befehl");
}

#[test]
fn real_packets_give_instructions_irqs_coverage_and_sid_writes() {
    let ps = packets(); let mut an = analyzer(); feed(&mut an, &ps);
    let s = an.summary();
    assert_eq!(s.cycles, 21778); assert_eq!(s.packets, 61); assert_eq!(s.lost_packets, 0);
    // Bad Lines: 43 Zyklen BA = 0, alle 504 Zyklen; Sprite-DMA kommt dazu
    assert!(s.stall_cycles > 400 && s.stall_cycles < 5000, "{}", s.stall_cycles);
    assert!(s.synced && s.insns > 4000, "{s:?}");
    assert!(s.irqs >= 4, "mindestens ein Raster-IRQ je Bild: {s:?}");
    assert_eq!(s.nmis, 0);
    // Warteschleife: LDA $0A89 / BEQ (0A95, 0A98); der Zähler in $0A89 wird später ungleich 0, dann läuft die Schleife aus und RTS (0A9A) wird ausgeführt.
    // Vorher ist 0A9A nur das Blindlesen der genommenen Verzweigung: Das Blindlesen allein darf nicht als Abruf zählen (siehe `blind_read_is_not_a_fetch`).
    assert!(an.is_covered(0x0A95) && an.is_covered(0x0A98) && an.is_covered(0x0A9A));
    assert!(!an.is_covered(0x0A96) && !an.is_covered(0x0A97), "Operandbytes sind keine Befehlsstarts");
    // Schreibzugriffe auf SID und VIC mit Zeitstempel
    let io = an.recent_io(100000);
    assert!(io.iter().any(|w| w.addr == 0xD400) && io.iter().any(|w| w.addr == 0xD021) && io.iter().any(|w| w.addr == 0xD019));
    assert!(io.windows(2).all(|w| w[0].cycle <= w[1].cycle));
    println!("{s:#?}");
}

#[test]
fn real_packets_loop_takes_seven_cycles_per_pass_between_bad_lines() {
    // LDA abs (4) + BEQ genommen (3) = 7 Zyklen je Durchlauf; zwischen A = $0A95 und dem nächsten Zugriff darauf: 7
    let ps = packets(); let mut an = analyzer();
    let id = an.add_measure(MeasureSpec { name: "Schleife".into(), from: 0x0A95, to: 0x0A95 });
    feed(&mut an, &ps);
    let m = an.measure(id).unwrap();
    assert!(m.completed > 100, "{} Durchläufe", m.completed);
    // Der IRQ-Einsprung liest den Opcode zweimal hintereinander (Zyklus 1 und 2): ein Zugriff auf A im Abstand von 1 Zyklus. Das ist die Wahrheit des Busses.
    assert_eq!(m.min, Some(1), "IRQ-Einsprung");
    assert!(m.values.iter().filter(|&&v| v == 1).count() <= 12, "nur wenige IRQs in 61 Paketen");
    // Die größeren Werte sind die Durchläufe mit BA-Pause (+43 Wiederholungen) oder dem IRQ
    assert!(m.values.iter().all(|&v| v >= 7 || v == 1));
    assert!(m.values.contains(&50), "7 + 43 Zyklen Bad Line");
    assert!(m.values.iter().filter(|&&v| v == 7).count() > m.values.len() / 2);
}

#[test]
fn real_packets_with_a_lost_packet_restart_without_guessing() {
    let ps = packets();
    let mut an = analyzer();
    let id = an.add_measure(MeasureSpec { name: "x".into(), from: 0x0A95, to: 0x0A95 });
    for (i, p) in ps.iter().enumerate() {
        if i == 10 { continue; }
        let seq = u16::from_le_bytes([p[0], p[1]]);
        an.feed(p, &RxMeta { seq, lost_before: (i == 11) as u16, late: false });
    }
    let s = an.summary();
    assert_eq!(s.lost_packets, 1); assert_eq!(s.epochs, 1); assert!(s.synced);
    assert!(an.measure(id).unwrap().invalid >= 1 || an.measure(id).unwrap().completed > 0);
    // Zeitstempel einer neuen Epoche fangen bei 0 an: die Epoche steht im Eintrag
    let rec = an.recent(1_000_000);
    assert!(rec.iter().any(|r| r.epoch == 1) && rec.iter().any(|r| r.epoch == 0));
}

#[test]
fn trace_lines_name_the_source() {
    let d = vc_ide::tests::static_checks::Dbg::parse(r#"{"format":1,"symbols":[{"name":"warte","value":2709}],"segments":[],"cycleBlocks":[],"lines":[{"file":"/p/x.asm","line":12,"addr":2709,"len":3,"cycles":[4,4]},{"file":"/p/x.asm","line":13,"addr":2712,"len":2,"cycles":[2,4]},{"file":"/p/x.asm","line":20,"addr":4096,"len":1,"cycles":[2,2]}]}"#).unwrap();
    let map = LineMap::from_dbg(&d);
    let ps = packets(); let mut an = analyzer();
    an.set_entry_points(map.entry_points());
    feed(&mut an, &ps);
    let r = an.recent(10).into_iter().find(|r| r.pc == 0x0A95 && r.kind == InsnKind::Op).unwrap();
    let line = format_record(&r, Some(&map));
    assert!(line.contains("$0A95") && line.contains("x.asm:12"), "{line}");
    let un = map.uncovered(|a| an.is_covered(a));
    assert_eq!(un, vec![("/p/x.asm".to_string(), 20, 0x1000)], "nur die nie ausgeführte Zeile");
    assert_eq!(map.symbol_for(2711).as_deref(), Some("warte+2"));
}

/// Hardware: Debug-Strom starten, Ablauf messen, stoppen. Die Ultimate muss im Debug-Modus „6510 Only“ stehen und ein Programm laufen lassen.
#[test]
#[ignore]
fn hardware_trace_probe() {
    let host = std::env::var("U64_HOST").expect("U64_HOST");
    let mut an = analyzer();
    let id = an.add_measure(MeasureSpec { name: "Schleife".into(), from: 0x0A95, to: 0x0A95 });
    let mut s = vc_ide::trace::TraceSession::start(&host, None, vc_ide::streams::StreamOptions::default(), an).expect("Start");
    let t0 = std::time::Instant::now();
    while t0.elapsed() < std::time::Duration::from_secs(3) { std::thread::sleep(std::time::Duration::from_millis(100)); s.poll(); }
    let an = s.analyzer().clone();
    let sum = s.stop();
    println!("{sum:#?}\n{:?}", an.lock().unwrap().measure(id));
    assert!(sum.packets > 0, "keine Pakete: Firewall für das Testprogramm freigeben oder `vcc streams trace` benutzen");
    assert!(sum.cycles > 0);
}

#[test]
fn measure_cycles_reports_unreachable_device() {
    let t0 = std::time::Instant::now();
    let r = vc_ide::trace::measure_cycles("127.0.0.1:1", None, vc_ide::streams::StreamOptions::default(), 0x1000, 0x1010, std::time::Duration::from_millis(200), || {});
    assert!(r.is_err_and(|e| e.contains("nicht erreichbar")));
    assert!(t0.elapsed() < std::time::Duration::from_secs(20));
}
