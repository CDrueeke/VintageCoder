// SPDX-License-Identifier: GPL-3.0-or-later
//! Ablaufprotokoll, Zyklenmessung und Abdeckung auf der Ultimate 64 (Debug-Strom, Modus „6510 Only“).
//!
//! Die Auswertung selbst steht in `vc_emu::streams::trace` (im Empfangsthread, ohne Stau). Hier hängen die Teile der IDE daran:
//! die Opcode-Tabelle aus der CPU-Beschreibung, die Zuordnung der Adressen zu Quellzeilen über die `.vcdbg`, die Strom-Sitzung
//! (Dienst, der den Bildstrom vorher stoppt) und die Messung „Zyklen von Adresse A bis B“ für Hardware-Testläufe.
//!
//! Das Ergebnis einer Messung auf der Hardware wird in `vcc test` (Modus `measure`, `tests::cycles`) gegen ein Zyklenbudget gewertet;
//! bei Paketverlust ist die Messung ungültig und wird nie geschätzt.
use crate::streams::{StreamOptions, StreamService};
use crate::tests::static_checks::Dbg;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use vc_emu::streams::trace::{analyzer_sink, shared, Analyzer, InsnKind, InsnRec, MeasureResult, MeasureSpec, OpTable, SharedAnalyzer, Summary};
use vc_emu::streams::StreamKind;
use vc_target::Cpu;

/// Opcode-Tabelle (Länge, Zyklen, Seitenübertrag) aus der CPU-Beschreibung, z. B. `6510`
pub fn op_table(cpu: &Cpu) -> OpTable { OpTable::from_fn(|op| cpu.by_opcode(op).map(|o| (o.bytes, o.cycles, o.page_cross))) }

/// Adresse → Quellzeile aus der `.vcdbg`
#[derive(Debug, Clone, Default)]
pub struct LineMap { by_addr: HashMap<u16, (String, usize)>, symbols: Vec<(u16, String)> }

impl LineMap {
    pub fn from_dbg(d: &Dbg) -> LineMap {
        let mut m = LineMap::default();
        for l in d.lines.iter().filter(|l| l.cycles.is_some()) { m.by_addr.entry(l.addr).or_insert_with(|| (l.file.clone(), l.line)); }
        m.symbols = d.symbols.iter().filter(|(_, &v)| (0..=0xFFFF).contains(&v)).map(|(n, &v)| (v as u16, n.clone())).collect();
        m.symbols.sort();
        m
    }
    pub fn line(&self, addr: u16) -> Option<(&str, usize)> { self.by_addr.get(&addr).map(|(f, l)| (f.as_str(), *l)) }
    /// Adressen der Befehle: die „bekannten Einstiege“ für die Befehlserkennung
    pub fn entry_points(&self) -> impl Iterator<Item = u16> + '_ { self.by_addr.keys().copied() }
    /// Zeilen mit Befehl, deren Adresse nie als Opcode abgerufen wurde (Datei, Zeile, Adresse), sortiert
    pub fn uncovered(&self, covered: impl Fn(u16) -> bool) -> Vec<(String, usize, u16)> {
        let mut v: Vec<_> = self.by_addr.iter().filter(|(a, _)| !covered(**a)).map(|(a, (f, l))| (f.clone(), *l, *a)).collect();
        v.sort(); v
    }
    /// Symbol, das am nächsten unterhalb der Adresse liegt (`name` oder `name+4`)
    pub fn symbol_for(&self, addr: u16) -> Option<String> {
        let i = self.symbols.partition_point(|(a, _)| *a <= addr);
        let (a, n) = self.symbols[..i].iter().rev().find(|(a, _)| addr - a < 0x100)?;
        Some(if *a == addr { n.clone() } else { format!("{n}+{}", addr - a) })
    }
}

/// Eine Zeile des Ablaufprotokolls: `Zyklus  $PC  Zyklen  Quelle`
pub fn format_record(r: &InsnRec, map: Option<&LineMap>) -> String {
    let what = match r.kind { InsnKind::Op => String::new(), InsnKind::Irq => "  IRQ".into(), InsnKind::Nmi => "  NMI".into() };
    let src = map.and_then(|m| m.line(r.pc).map(|(f, l)| format!("  {}:{l}", std::path::Path::new(f).file_name().map_or(f.to_string(), |n| n.to_string_lossy().into_owned()))).or_else(|| m.symbol_for(r.pc).map(|s| format!("  {s}")))).unwrap_or_default();
    let stall = if r.stall > 0 { format!(" (+{} BA)", r.stall) } else { String::new() };
    format!("{:>10}  ${:04X}  {}{}{}{}", r.start, r.pc, r.cycles, stall, what, src)
}

/// Strom-Sitzung des Ablaufprotokolls: Dienst und Analysator zusammen. Der Bildstrom wird beim Start gestoppt.
pub struct TraceSession {
    svc: StreamService,
    an: SharedAnalyzer,
}

impl TraceSession {
    /// Debug-Strom starten und jedes Paket im Empfangsthread auswerten. Kehrt nach dem REST-Aufruf zurück.
    pub fn start(host: &str, password: Option<&str>, opts: StreamOptions, analyzer: Analyzer) -> Result<TraceSession, String> {
        let an = shared(analyzer);
        let mut svc = StreamService::new(host, password, opts);
        svc.start(StreamKind::Debug, analyzer_sink(&an))?;
        Ok(TraceSession { svc, an })
    }
    pub fn analyzer(&self) -> &SharedAnalyzer { &self.an }
    pub fn service(&mut self) -> &mut StreamService { &mut self.svc }
    pub fn poll(&mut self) { self.svc.poll(); }
    pub fn summary(&self) -> Summary { self.an.lock().map(|a| a.summary()).unwrap_or_default() }
    pub fn stop(mut self) -> Summary { self.svc.stop_all(); self.summary() }
}

/// Zyklen von Adresse `from` bis `to` auf der Hardware messen: Debug-Strom starten, warten, bis eine gültige Messung vorliegt (oder die Zeit
/// abläuft), Strom stoppen. Der Aufrufer startet das Programm danach oder davor (die Messung beginnt am nächsten Zugriff auf `from`).
/// `Ok` mit `completed == 0`: nichts gemessen (Programm lief nicht durch oder Pakete gingen verloren, `invalid`).
pub fn measure_cycles(host: &str, password: Option<&str>, opts: StreamOptions, from: u16, to: u16, timeout: Duration, on_started: impl FnOnce()) -> Result<MeasureResult, String> {
    measure_cycles_with(host, password, opts, from, to, timeout, || { on_started(); Ok(()) }).map(|m| m.0)
}

/// Wie [`measure_cycles`], aber `on_started` darf scheitern (dann endet die Messung sofort mit dem Fehler, der Strom wird gestoppt) und die
/// Zusammenfassung des Stroms kommt mit zurück (Pakete, Verluste), damit „keine Pakete“ von „Programm erreicht B nicht“ unterscheidbar bleibt.
pub fn measure_cycles_with(host: &str, password: Option<&str>, opts: StreamOptions, from: u16, to: u16, timeout: Duration, on_started: impl FnOnce() -> Result<(), String>) -> Result<(MeasureResult, Summary), String> {
    let mut an = Analyzer::new(OpTable::empty());       // für die Messung zählt nur die Zyklenzahl; Befehlsgrenzen braucht sie nicht
    let id = an.add_measure(MeasureSpec { name: format!("${from:04X}..${to:04X}"), from, to });
    an.set_measure_reads_only(true);                    // das Laden des Programms erscheint als Schreibzyklen an dessen Adressen
    an.set_restart_on_reset(true);                      // der Reset beim Programmstart verwirft Zugriffe des vorigen Programms
    let mut s = TraceSession::start(host, password, opts, an)?;
    if let Err(e) = on_started() { s.stop(); return Err(e); }
    let t0 = Instant::now();
    let r = loop {
        std::thread::sleep(Duration::from_millis(50)); s.poll();
        let m = s.an.lock().map_err(|_| "Auswertung abgestürzt".to_string())?.measure(id).ok_or("keine Messung")?;
        if m.completed > 0 || t0.elapsed() > timeout { break m; }
        if let crate::streams::StreamState::Error(e) = s.svc.state(StreamKind::Debug) { let e = e.clone(); s.svc.stop_all(); return Err(e); }
    };
    let sum = s.stop();
    if let Some(u) = &sum.unsupported { return Err(u.clone()); }
    Ok((r, sum))
}

/// Aufzeichnung des Ablaufprotokolls in eine Textdatei: ein Schreib-Thread, der Befehle aus einem begrenzten Kanal holt
pub struct TraceWriter { pub tx: std::sync::mpsc::SyncSender<Vec<InsnRec>>, handle: std::thread::JoinHandle<Result<u64, String>> }

impl TraceWriter {
    pub fn create(path: &std::path::Path, map: Option<LineMap>) -> Result<TraceWriter, String> {
        use std::io::Write;
        let f = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<InsnRec>>(256);
        let handle = std::thread::Builder::new().name("vc-trace-writer".into()).spawn(move || {
            let mut w = std::io::BufWriter::new(f); let mut n = 0u64; let mut epoch = 0u32;
            writeln!(w, "# Zyklus  PC  Zyklen  Quelle (Zyklen zählen je Epoche; nach einem Paketverlust beginnt eine neue)").map_err(|e| e.to_string())?;
            for batch in rx {
                for r in batch {
                    if r.epoch != epoch { epoch = r.epoch; writeln!(w, "# --- Paketverlust: neue Epoche {epoch} ---").map_err(|e| e.to_string())?; }
                    writeln!(w, "{}", format_record(&r, map.as_ref())).map_err(|e| e.to_string())?; n += 1;
                }
            }
            w.flush().map_err(|e| e.to_string())?;
            Ok(n)
        }).map_err(|e| e.to_string())?;
        Ok(TraceWriter { tx, handle })
    }
    /// Kanal schließen und warten; liefert die Zahl geschriebener Befehle
    pub fn finish(self) -> Result<u64, String> { drop(self.tx); self.handle.join().map_err(|_| "Schreib-Thread abgestürzt".to_string())? }
}

