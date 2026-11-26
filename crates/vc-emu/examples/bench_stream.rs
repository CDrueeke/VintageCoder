// SPDX-License-Identifier: GPL-3.0-or-later
//! Messung des Bildstroms gegen echtes VICE: Bildrate, Takt, Verzögerung, Auslassungen, Emulationstempo, CPU und Speicher.
//! Zusätzliche VICE-Argumente über `VC_BENCH_ARGS` (z. B. `+sound`).
//! Aufruf: `cargo run --release --example bench_stream -- <programm.prg> <rate> <sekunden> [deliver_unchanged]`
use std::process::Command;
use std::time::{Duration, Instant};
use vc_emu::framestream::{share, FrameStream, NextError, StreamConfig};
use vc_emu::{Backend, Vice};

/// (CPU-Sekunden, Speicher in MB) der Prozesse, deren Kommandozeile `needle` enthält
fn ps(needle: &str, pid: Option<u32>) -> (f64, f64) {
    let out = Command::new("ps").args(["-axo", "pid=,cputime=,rss=,command="]).output().unwrap();
    let (mut cpu, mut rss) = (0.0, 0.0);
    for l in String::from_utf8_lossy(&out.stdout).lines() {
        let mut it = l.split_whitespace();
        let (p, t, r) = (it.next().unwrap_or(""), it.next().unwrap_or(""), it.next().unwrap_or(""));
        let cmd = l.splitn(4, char::is_whitespace).last().unwrap_or("");
        if pid.map_or(cmd.contains(needle), |x| p == x.to_string()) {
            let secs: f64 = t.split(':').fold(0.0, |a, x| a * 60.0 + x.parse::<f64>().unwrap_or(0.0));
            cpu += secs; rss += r.parse::<f64>().unwrap_or(0.0) / 1024.0;
        }
    }
    (cpu, rss)
}
fn jiffies(b: &mut dyn Backend) -> u32 { let m = b.read_mem(0xA0, 3).unwrap(); m[0] as u32 * 65536 + m[1] as u32 * 256 + m[2] as u32 }

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (prg, rate, secs) = (a[1].clone(), a[2].parse::<f64>().unwrap(), a[3].parse::<u64>().unwrap());
    let unchanged = a.get(4).map_or(false, |x| x == "unchanged");
    let extra: Vec<String> = std::env::var("VC_BENCH_ARGS").unwrap_or_default().split_whitespace().map(String::from).collect();
    let mut v = Vice::launch_with("x64sc", &prg, None, &extra).expect("VICE");
    v.tempo_hold_from_env();
    let be = share(Box::new(v));
    std::thread::sleep(Duration::from_secs(3));
    let (j0, t0) = { let mut g = be.lock().unwrap(); let j = jiffies(&mut **g); g.cont().unwrap(); (j, Instant::now()) };
    let me = std::process::id();
    let (c0, _) = ps("", Some(me));
    let (vc0, _) = ps("binarymonitoraddress", None);
    let s = FrameStream::start(be.clone(), StreamConfig { rate, deliver_unchanged: unchanged }).unwrap();
    let (mut lat, mut gaps, mut n, mut dropped, mut last_ts) = (vec![], vec![], 0u64, 0u64, None::<Duration>);
    let stream_start = Instant::now();
    let mut max_rss_me = 0.0f64;
    while stream_start.elapsed() < Duration::from_secs(secs) {
        match s.next(Duration::from_millis(500)) {
            Ok(f) => {
                // Verzögerung: Zeit zwischen Ende des Abrufs und Ankunft beim Verbraucher
                lat.push((stream_start.elapsed().as_secs_f64() - f.timestamp.as_secs_f64()) * 1000.0);
                if let Some(l) = last_ts { gaps.push((f.timestamp - l).as_secs_f64() * 1000.0); }
                last_ts = Some(f.timestamp); n += 1; dropped += f.dropped as u64;
            }
            Err(NextError::Timeout) => {}
            Err(e) => { eprintln!("Ende: {e:?}"); break; }
        }
        if n % 100 == 0 { max_rss_me = max_rss_me.max(ps("", Some(me)).1); }
    }
    let wall = stream_start.elapsed().as_secs_f64();
    let st = s.stop();
    let (c1, rss_me) = ps("", Some(me));
    let (vc1, rss_v) = ps("binarymonitoraddress", None);
    let (j1, t1) = { let mut g = be.lock().unwrap(); let j = jiffies(&mut **g); (j, Instant::now()) };
    let tempo = (j1 - j0) as f64 / 60.2 / (t1 - t0).as_secs_f64();
    gaps.sort_by(|x, y| x.partial_cmp(y).unwrap()); lat.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let pct = |v: &Vec<f64>, p: f64| if v.is_empty() { f64::NAN } else { v[((v.len() - 1) as f64 * p) as usize] };
    let mean = gaps.iter().sum::<f64>() / gaps.len().max(1) as f64;
    let dev = (gaps.iter().map(|g| (g - mean).powi(2)).sum::<f64>() / gaps.len().max(1) as f64).sqrt();
    println!("Ziel {rate} B/s, {wall:.1} s{}: geliefert {n} = {:.1} B/s, ausgelassen {dropped} (Strom: {} + {} unverändert, Fehler {})", if unchanged { " (auch unveränderte)" } else { "" }, n as f64 / wall, st.delivered, st.unchanged, st.errors);
    println!("  Abstand: Mittel {mean:.2} ms, Streuung {dev:.2} ms, p50 {:.1}, p95 {:.1}, p99 {:.1}, max {:.1} ms", pct(&gaps, 0.5), pct(&gaps, 0.95), pct(&gaps, 0.99), pct(&gaps, 1.0));
    println!("  Verzögerung Abruf→Verbraucher: p50 {:.2} ms, p95 {:.2} ms, max {:.2} ms", pct(&lat, 0.5), pct(&lat, 0.95), pct(&lat, 1.0));
    println!("  Abruf (VICE steht): Mittel {:.2} ms, max {:.2} ms", st.mean_grab_ms, st.max_grab_ms);
    println!("  Emulationstempo (Jiffies): {:.0} % (Jiffy-Takt 60,2 Hz = 100 %)", tempo * 100.0);
    println!("  CPU: VintageCoder {:.1} %, VICE {:.1} % (je ein Kern); Speicher: VintageCoder {:.0} MB, VICE {:.0} MB", (c1 - c0) / wall * 100.0, (vc1 - vc0) / wall * 100.0, rss_me.max(max_rss_me), rss_v);
    be.lock().unwrap().stop();
}
