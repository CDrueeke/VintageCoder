// SPDX-License-Identifier: GPL-3.0-or-later
//! Tracker: Quelle des Tons (Synthesizer | Ultimate), „Auf Hardware hören“, WAV-Aufnahme und Vergleich der Aufnahme mit dem Synthesizer.
//! Der Strom startet nur auf ausdrückliche Handlung (Knopf mit Bestätigung) und wird beim Stoppen oder Schließen des Fensters beendet.
use crate::audio::{Audio, Source};
use crate::synth::{Engine, SongData};
use crate::ui::{dialog, tokens::{t, tint}, widgets};
use crate::viz;
use eframe::egui::{self, pos2, vec2, Color32, RichText, Sense, Stroke};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use vc_emu::streams::{audio::{RATE_NTSC, RATE_PAL}, wav, StreamKind};
use vc_ide::streams::{StreamService, StreamState};

/// Angaben des Lieds, die der Streifen braucht
pub struct SongInfo<'a> { pub root: &'a Path, pub data: &'a Path, pub name: &'a str, pub profile: &'a str, pub chip: &'a str, pub channels: usize, pub hz: f32, pub json: &'a dyn Fn() -> String, pub song: &'a dyn Fn() -> SongData }

type Job = mpsc::Receiver<Result<Option<StreamService>, String>>;

/// Ergebnis des Vergleichs von Aufnahme (B) und Synthesizer (A)
#[derive(Debug)]
pub struct Compare {
    pub rate: f64, pub file: String, pub cmp: Option<wav::Comparison>,
    /// gemittelte Spektren und Pegel (RMS, Spitze in dBFS)
    pub spec_a: [f32; viz::BANDS], pub spec_b: [f32; viz::BANDS], pub lv_a: (f64, f64), pub lv_b: (f64, f64),
}

pub struct ListenPanel {
    svc: Arc<Mutex<Option<StreamService>>>, job: Option<Job>, stopping: bool,
    confirm: bool, status: String, error: Option<String>,
    machine: Option<(String, String, String)>, machine_at: Option<std::time::Instant>,
    last_rec: Option<PathBuf>,
    cmp_job: Option<mpsc::Receiver<Result<Compare, String>>>, pub cmp: Option<Compare>, cmp_open: bool, cmp_view: String,
    demo: bool,
}
impl Default for ListenPanel {
    fn default() -> Self {
        ListenPanel { svc: Default::default(), job: None, stopping: false, confirm: false, status: String::new(), error: None, machine: None, machine_at: None, last_rec: None, cmp_job: None, cmp: None, cmp_open: false, cmp_view: "both".into(), demo: false }
    }
}

/// Zeit für Dateinamen (`JJJJMMTT-HHMMSS`, Ortszeit über `date` bzw. unter Windows PowerShell `Get-Date`; ohne beides: UTC)
pub fn stamp() -> String {
    #[cfg(windows)]
    let out = { use std::os::windows::process::CommandExt; std::process::Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", "Get-Date -Format yyyyMMdd-HHmmss"]).creation_flags(0x0800_0000).output() };
    #[cfg(not(windows))]
    let out = std::process::Command::new("date").arg("+%Y%m%d-%H%M%S").output();
    if let Ok(o) = out {
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if o.status.success() && s.len() == 15 { return s; }
    }
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let z = days + 719468; let era = z.div_euclid(146097); let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153; let d = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Dateiname einer Aufnahme: `aufnahmen/<lied>_<zeit>.wav`
pub fn recording_path(root: &Path, song: &str) -> PathBuf {
    let clean: String = song.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    root.join("aufnahmen").join(format!("{}_{}.wav", if clean.is_empty() { "lied" } else { &clean }, stamp()))
}

/// Mittleres Spektrum über den überlappenden Teil beider Signale (`shift`: Index in `b` = Index in `a` + shift)
pub fn avg_spectrum(a: &[f32], b: &[f32], shift: i64, rate: f32) -> ([f32; viz::BANDS], [f32; viz::BANDS]) {
    let (i0, i1) = (0.max(-shift), (a.len() as i64).min(b.len() as i64 - shift));
    let (mut sa, mut sb, mut n) = ([0.0f32; viz::BANDS], [0.0f32; viz::BANDS], 0);
    if i1 - i0 >= viz::FFT_N as i64 {
        let k = 24; let span = i1 - i0 - viz::FFT_N as i64;
        for j in 0..k {
            let s = i0 + span * j / (k - 1);
            let (wa, wb) = (&a[..(s as usize + viz::FFT_N)], &b[..((s + shift) as usize + viz::FFT_N)]);
            let (xa, xb) = (viz::spectrum(wa, rate), viz::spectrum(wb, rate));
            for q in 0..viz::BANDS { sa[q] += xa[q]; sb[q] += xb[q]; }
            n += 1;
        }
    }
    if n > 0 { for q in 0..viz::BANDS { sa[q] /= n as f32; sb[q] /= n as f32; } }
    (sa, sb)
}

fn peak_db(x: &[f32]) -> f64 { 20.0 * (x.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-6) as f64).log10() }

/// Synthesizer-Signal des Lieds offline in `rate` rendern (Mono, `secs` Sekunden)
pub fn render_synth(song: SongData, rate: f32, secs: f32) -> Vec<f32> {
    let mut e = Engine::default(); e.play(song, 0, 0);
    let mut out = vec![0.0f32; (rate * secs) as usize];
    for c in out.chunks_mut(4096) { e.render(c, rate); }
    out
}

/// Aufnahme laden, Synthesizer rendern, ausrichten und vergleichen (läuft in einem Hilfsthread)
pub fn build_compare(song: SongData, wav_path: &Path) -> Result<Compare, String> {
    let bytes = std::fs::read(wav_path).map_err(|e| format!("{}: {e}", wav_path.display()))?;
    let (rate, frames) = wav::read_wav(&bytes)?;
    let mut rec = wav::mono(&frames);
    let max = rate as usize * 30; rec.truncate(max);            // Vergleich über höchstens 30 s
    if rec.len() < rate as usize / 2 { return Err("Die Aufnahme ist zu kurz zum Vergleichen (unter einer halben Sekunde).".into()); }
    let syn = render_synth(song, rate as f32, rec.len() as f32 / rate as f32 + 10.0);
    let cmp = wav::compare(&syn, &rec, rate as f64, 10.0);
    let shift = cmp.map_or(0, |c| (c.lag_secs * rate as f64).round() as i64);
    let (spec_a, spec_b) = avg_spectrum(&syn, &rec, shift, rate as f32);
    let (i0, i1) = (0.max(-shift), (syn.len() as i64).min(rec.len() as i64 - shift));
    let (sa, sb): (&[f32], &[f32]) = if i1 > i0 { (&syn[i0 as usize..i1 as usize], &rec[(i0 + shift) as usize..(i1 + shift) as usize]) } else { (&[], &[]) };
    Ok(Compare { rate: rate as f64, file: wav_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), cmp, spec_a, spec_b, lv_a: (cmp.map_or(f64::NEG_INFINITY, |c| c.rms_a_db), peak_db(sa)), lv_b: (cmp.map_or(f64::NEG_INFINITY, |c| c.rms_b_db), peak_db(sb)) })
}

fn ultimate_machine() -> Option<(String, String, String)> {
    vc_ide::settings::effective().machines.iter().find(|m| m.kind == "ultimate64" && !m.host.trim().is_empty()).map(|m| (m.display_name().to_string(), m.host.trim().to_string(), m.password.clone()))
}

impl ListenPanel {
    pub fn running(&self) -> bool { self.demo || self.svc.lock().map_or(false, |g| g.is_some()) }
    fn busy(&self) -> bool { self.job.is_some() }

    /// Selbsttest (`VC_SHOT`): Quelle „Ultimate“ mit echten Testpaketen statt Hardware, Strom als laufend gemeldet
    pub fn start_demo(&mut self, audio: &Audio) {
        if self.demo { return; }
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../vc-emu/tests/data/audio.bin");
        let Ok(b) = std::fs::read(&p) else { self.error = Some(format!("{}: nicht lesbar", p.display())); return };
        let (mut o, mut ps) = (0, vec![]);
        while o + 2 <= b.len() { let n = u16::from_le_bytes([b[o], b[o + 1]]) as usize; ps.push(b[o + 2..o + 2 + n].to_vec()); o += 2 + n; }
        audio.set_source(Source::Ultimate); self.demo = true;
        let hub = audio.hub();
        std::thread::spawn(move || {
            let t0 = std::time::Instant::now(); let mut sent = 0u64;
            loop {
                let due = (t0.elapsed().as_secs_f64() * 250.0) as u64;
                while sent < due { let mut p = ps[(sent as usize) % ps.len()].clone(); p[0] = sent as u8; p[1] = (sent >> 8) as u8; if let Ok(mut h) = hub.lock() { h.on_packet(&p, vc_emu::streams::RxMeta { seq: sent as u16, lost_before: 0, late: false }); } sent += 1; }
                std::thread::sleep(std::time::Duration::from_millis(4));
                if t0.elapsed().as_secs() > 600 { break; }
            }
        });
    }

    /// Strom und Gerät beenden (im Hintergrund)
    fn stop(&mut self, audio: &Audio, reset: bool) {
        if let Some((p, s)) = audio.stop_recording() { self.status = format!("Aufnahme gespeichert: {} ({s:.1} s)", p.display()); self.last_rec = Some(p); }
        audio.set_source(Source::Synth);
        let svc = self.svc.clone(); self.stopping = true;
        let (tx, rx) = mpsc::channel(); self.job = Some(rx);
        std::thread::spawn(move || {
            let taken = svc.lock().ok().and_then(|mut g| g.take());
            let r = match taken { Some(mut s) => vc_ide::listen::stop_hardware(&mut s, reset).map(|_| None), None => Ok(None) };
            let _ = tx.send(r);
        });
    }

    fn start(&mut self, audio: &Audio, info: &SongInfo) {
        let Some((_, host, pw)) = self.machine.clone() else { return };
        let ntsc = info.hz > 55.0;
        let prg = match vc_ide::listen::build_player(info.data, &(info.json)(), ntsc) { Ok(p) => p, Err(e) => { self.error = Some(e); return } };
        let st = vc_ide::settings::effective();
        let opts = vc_ide::streams::StreamOptions::from_settings(&st);
        audio.set_source(Source::Ultimate);
        if let Ok(mut h) = audio.hub().lock() { h.player = vc_emu::streams::audio::StreamPlayer::new(if ntsc { RATE_NTSC } else { RATE_PAL }, 48000.0); }
        let hub = audio.hub(); self.error = None; self.status = "Programm wird geladen …".into();
        let (tx, rx) = mpsc::channel(); self.job = Some(rx); self.stopping = false;
        std::thread::spawn(move || { let _ = tx.send(vc_ide::listen::start_hardware(&host, Some(&pw), opts, &prg, hub).map(Some)); });
    }

    fn poll(&mut self, audio: &Audio) {
        if let Some(rx) = &self.job {
            match rx.try_recv() {
                Ok(r) => {
                    self.job = None;
                    match r {
                        Ok(Some(s)) => { if let Ok(mut g) = self.svc.lock() { *g = Some(s); } self.status = "Hörprobe läuft".into(); }
                        Ok(None) => { self.status = if self.stopping { "Hörprobe beendet".into() } else { String::new() }; }
                        Err(e) => { self.error = Some(e); audio.set_source(Source::Synth); self.status.clear(); }
                    }
                    self.stopping = false;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(_) => { self.job = None; }
            }
        }
        if let Some(rx) = &self.cmp_job {
            match rx.try_recv() { Ok(r) => { self.cmp_job = None; match r { Ok(c) => { self.cmp = Some(c); self.cmp_open = true; } Err(e) => self.error = Some(e) } } Err(mpsc::TryRecvError::Empty) => {} Err(_) => self.cmp_job = None }
        }
        if let Ok(mut g) = self.svc.try_lock() {
            if let Some(s) = g.as_mut() {
                s.poll();
                if let StreamState::Error(e) = s.state(StreamKind::Audio).clone() { self.error = Some(e); *g = None; drop(g); audio.set_source(Source::Synth); self.status.clear(); }
            }
        }
        if let Some(e) = audio.take_recording_error() { self.error = Some(format!("Aufnahme abgebrochen: {e}")); }
    }

    fn start_compare(&mut self, info: &SongInfo, path: PathBuf) {
        let song = (info.song)(); let (tx, rx) = mpsc::channel(); self.cmp_job = Some(rx); self.error = None;
        std::thread::spawn(move || { let _ = tx.send(build_compare(song, &path)); });
    }

    /// Streifen unter der Werkzeugleiste: Quelle, Strom-Status, Hörprobe, Aufnahme, Vergleich
    pub fn show(&mut self, ui: &mut egui::Ui, audio: &Audio, info: &SongInfo) {
        let k = t();
        if self.machine_at.map_or(true, |a| a.elapsed().as_secs() > 3) { self.machine = ultimate_machine(); self.machine_at = Some(std::time::Instant::now()); }
        if std::env::var("VC_SHOT").is_ok() && std::env::var("VC_LISTEN_DEMO").is_ok() { self.start_demo(audio); if let Ok(p) = std::env::var("VC_LISTEN_WAV") { if self.cmp.is_none() && self.cmp_job.is_none() && self.last_rec.is_none() { self.last_rec = Some(PathBuf::from(p.clone())); self.start_compare(info, PathBuf::from(p)); } } }
        self.poll(audio);
        let reason = vc_ide::listen::unsupported_reason(info.profile, info.chip, info.channels)
            .or_else(|| if self.machine.is_none() && !self.demo { Some("Es ist keine Ultimate eingerichtet (Einstellungen → Maschinen).".to_string()) } else { None });
        let running = self.running(); let ult = audio.source() == Source::Ultimate;
        let mut src = if ult { "ult".to_string() } else { "synth".to_string() };
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Quelle").strong().color(k.mute));
            if widgets::segmented(ui, &mut src, &[("synth", "Synthesizer"), ("ult", "Ultimate")]) {
                if src == "ult" { if reason.is_none() { audio.set_source(Source::Ultimate); } else { self.error = reason.clone(); } }
                else if running { self.stop(audio, true); } else { audio.set_source(Source::Synth); }
            }
            if let Some(r) = &reason { ui.label(RichText::new("Hardware nicht wählbar").small().color(k.am)).on_hover_text(r.as_str()); }
            else if let Some((name, host, _)) = &self.machine { ui.label(RichText::new(format!("{name} · {host}")).small().color(k.mute)); }
            if running {
                let (mbit, lost, state) = if self.demo { (1.5, 0.0, StreamState::Running) } else {
                    self.svc.try_lock().ok().and_then(|g| g.as_ref().map(|s| { let st = s.status(StreamKind::Audio); (st.mbit_per_s, st.loss_percent, st.state) })).unwrap_or((0.0, 0.0, StreamState::Starting))
                };
                let (txt, col) = match state { StreamState::Running => ("läuft", k.gr), StreamState::Lossy => ("Verluste", if lost >= 5.0 { k.rd } else { k.am }), StreamState::Starting => ("startet …", k.am), _ => ("aus", k.mute) };
                widgets::chip(ui, &format!("Ton {txt}"), col);
                let b = audio.buffer_stats();
                ui.label(RichText::new(format!("{mbit:.1} Mbit/s · {lost:.1} % Verluste · Puffer {:.0} ms", b.fill_ms)).small().color(k.mute));
            }
            ui.separator();
            let can_listen = reason.is_none() && !self.busy();
            if running {
                if ui.add_enabled(!self.busy(), egui::Button::new(RichText::new("■ Stopp").strong().color(Color32::WHITE)).fill(k.rd)).on_hover_text("Strom beenden und die Ultimate zurücksetzen").clicked() { if self.demo { self.demo = false; audio.set_source(Source::Synth); } else { self.stop(audio, true); } }
            } else if ui.add_enabled(can_listen, egui::Button::new(RichText::new("▶ Auf Hardware hören").strong().color(k.on_cy)).fill(k.cy)).on_hover_text(reason.clone().unwrap_or_else(|| "Lied als Programm auf die Ultimate laden und über den echten SID spielen; der Ton kommt als Datenstrom zurück".into())).clicked() { self.confirm = true; }
            let rec = audio.recording();
            if let Some((secs, _)) = &rec {
                if ui.add(egui::Button::new(RichText::new(format!("■ Aufnahme {secs:.0} s")).color(Color32::WHITE)).fill(k.rd)).on_hover_text("Aufnahme beenden und speichern").clicked() {
                    if let Some((p, s)) = audio.stop_recording() { self.status = format!("Aufnahme gespeichert: {} ({s:.1} s)", p.display()); self.last_rec = Some(p); }
                }
                ui.ctx().request_repaint();
            } else if ui.add_enabled(running && ult, egui::Button::new("● Aufnahme")).on_hover_text("Den Ton der Ultimate als WAV-Datei aufnehmen (Quellrate, 16 Bit stereo) – Ablage im Projekt unter aufnahmen/").clicked() {
                let p = recording_path(info.root, info.name);
                match audio.start_recording(&p, if info.hz > 55.0 { RATE_NTSC } else { RATE_PAL }.round() as u32) { Ok(()) => self.status = format!("Aufnahme läuft: {}", p.display()), Err(e) => self.error = Some(e) }
            }
            let cmp_btn = ui.add_enabled(self.cmp_job.is_none(), egui::Button::new("Vergleich mit Synthesizer"));
            if cmp_btn.on_hover_text("Aufnahme gegen das Signal des Synthesizers legen (Spektrum, Pegel, Zahlenmaß)").clicked() {
                let file = self.last_rec.clone().filter(|p| p.exists()).or_else(|| rfd::FileDialog::new().set_directory(info.root.join("aufnahmen")).add_filter("WAV-Aufnahme", &["wav"]).set_title("Aufnahme wählen").pick_file());
                if let Some(f) = file { self.last_rec = Some(f.clone()); self.start_compare(info, f); }
            }
            if self.cmp_job.is_some() { ui.spinner(); ui.label(RichText::new("vergleiche …").small().color(k.mute)); }
        });
        if let Some(e) = self.error.clone() {
            ui.horizontal_wrapped(|ui| { ui.label(RichText::new("Fehler").strong().color(k.rd)); ui.add(egui::Label::new(RichText::new(&e).color(k.rd)).wrap()); if ui.small_button("✕").clicked() { self.error = None; } });
        } else if !self.status.is_empty() { ui.label(RichText::new(&self.status).small().color(k.mute)); }
        if ult { ui.label(RichText::new("Der Strom trägt nur die gemischte Ausgabe; Stimmen, Noten und Stumm/Solo gibt es nur mit dem Synthesizer.").small().color(k.dim)); }
        if self.confirm { self.confirm_dialog(ui.ctx(), audio, info); }
        if self.cmp_open { self.compare_window(ui.ctx()); }
    }

    fn confirm_dialog(&mut self, ctx: &egui::Context, audio: &Audio, info: &SongInfo) {
        let k = t(); let (name, host) = self.machine.as_ref().map(|m| (m.0.clone(), m.1.clone())).unwrap_or_default();
        let chips = [("Ultimate".to_string(), k.cy)];
        let act = dialog::show(ctx, &dialog::Dlg::new("Lied auf der Ultimate spielen?", "Auf Hardware hören").sub(&format!("{name} · {host}")).chips(&chips), |ui| {
            ui.set_max_width(460.0);
            widgets::card(ui, false, |ui| { ui.label(RichText::new("Ultimate 64 · REST + UDP-Datenstrom").strong()); ui.label(RichText::new("Die Verbindung wird beim Start geprüft. Der Ton-Strom kommt über das LAN-Kabel der Ultimate zu diesem Rechner (nicht über WLAN).").small().color(k.dim)); });
            ui.add_space(6.0);
            egui::Frame::new().fill(tint(k.am)).stroke(Stroke::new(1.0, k.am.gamma_multiply(0.4))).corner_radius(4.0).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                ui.label(RichText::new("Achtung").strong().color(k.am));
                ui.label("Die Ultimate wird zurückgesetzt und lädt das Abspielprogramm; der laufende Zustand des Geräts geht verloren. Beim Stoppen wird sie erneut zurückgesetzt. Es wird nichts im Flash gespeichert und keine Einstellung geändert.");
                ui.label("Der Strom beginnt erst jetzt und endet beim Stoppen oder Schließen des Fensters.");
            });
            true
        });
        match act { dialog::Act::Primary => { self.confirm = false; self.start(audio, info); } dialog::Act::Secondary => self.confirm = false, dialog::Act::None => {} }
    }

    fn compare_window(&mut self, ctx: &egui::Context) {
        let k = t(); let Some(c) = &self.cmp else { self.cmp_open = false; return };
        let chips = [("A Synthesizer".to_string(), k.cy), ("B Aufnahme".to_string(), k.am)];
        let mut view = self.cmp_view.clone();
        let act = dialog::show(ctx, &dialog::Dlg { title: "Vergleich mit Synthesizer", sub: &c.file, chips: &chips, primary: "", secondary: "Schließen" }, |ui| {
            ui.set_min_width(520.0);
            widgets::segmented(ui, &mut view, &[("a", "A Synthesizer"), ("b", "B Aufnahme"), ("both", "Übereinander")]);
            ui.add_space(6.0);
            ui.label(RichText::new("Spektrum (Mittel über das Stück)").strong());
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width().max(480.0), 130.0), Sense::hover());
            let p = ui.painter_at(r);
            let draw_bars = |vals: &[f32; viz::BANDS]| { let mut st = viz::VizState::default(); st.spec = *vals; viz::spectrum_view(&p, r, &st, false); };
            match view.as_str() {
                "a" => draw_bars(&c.spec_a), "b" => draw_bars(&c.spec_b),
                _ => {
                    p.rect_filled(r, 3.0, k.bg0);
                    for (vals, col) in [(&c.spec_a, k.cy), (&c.spec_b, k.am)] {
                        let gw = r.width() / viz::BANDS as f32;
                        let pts: Vec<_> = (0..viz::BANDS).map(|b| pos2(r.left() + (b as f32 + 0.5) * gw, r.bottom() - 4.0 - vals[b] * (r.height() - 8.0))).collect();
                        p.add(egui::Shape::line(pts.clone(), Stroke::new(2.0, col)));
                        for q in pts { p.circle_filled(q, 2.5, col); }
                    }
                }
            }
            ui.add_space(6.0);
            ui.label(RichText::new("Pegel (Effektivwert, Spitze)").strong());
            for (lab, lv, col, on) in [("A", c.lv_a, k.cy, view != "b"), ("B", c.lv_b, k.am, view != "a")] {
                if !on { continue; }
                ui.horizontal(|ui| {
                    ui.label(RichText::new(lab).strong().color(col));
                    let (rr, _) = ui.allocate_exact_size(vec2(260.0, 10.0), Sense::hover());
                    viz::hbar(&ui.painter_at(rr), rr, viz::level_of(10f32.powf(lv.0 as f32 / 20.0)), Some(viz::level_of(10f32.powf(lv.1 as f32 / 20.0))), 26);
                    ui.label(RichText::new(format!("{:.1} dBFS · Spitze {:.1} dBFS", lv.0, lv.1)).monospace().small().color(k.mute));
                });
            }
            ui.add_space(6.0);
            match &c.cmp {
                Some(m) => {
                    egui::Grid::new("cmp-nums").num_columns(2).show(ui, |ui| {
                        ui.label("Versatz der Aufnahme"); ui.label(RichText::new(format!("{:+.2} s", m.lag_secs)).monospace()); ui.end_row();
                        ui.label("Korrelation der Hüllkurven"); ui.label(RichText::new(format!("{:.2}", m.corr)).monospace().color(if m.corr >= 0.8 { k.gr } else if m.corr >= 0.5 { k.am } else { k.rd })); ui.end_row();
                        ui.label("Pegelunterschied B − A"); ui.label(RichText::new(format!("{:+.1} dB", m.diff_db)).monospace()); ui.end_row();
                        ui.label("Verglichene Länge"); ui.label(RichText::new(format!("{:.1} s bei {:.0} Hz", m.overlap_secs, c.rate)).monospace()); ui.end_row();
                    });
                }
                None => { ui.label(RichText::new("Keine Ausrichtung möglich (Stille oder zu kurz).").color(k.am)); }
            }
            ui.add_space(4.0);
            ui.add(egui::Label::new(RichText::new("Die Zahlen sind eine Hilfe, keine Prüfung: Ausgerichtet wird über die Lautstärkeverläufe (auf etwa 10 ms genau); der echte SID klingt durch Filter und Chip-Eigenheiten anders als der Synthesizer.").small().color(k.dim)).wrap());
            true
        });
        self.cmp_view = view;
        if act != dialog::Act::None { self.cmp_open = false; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stamp_has_date_and_time() { let s = stamp(); assert_eq!(s.len(), 15); assert_eq!(&s[8..9], "-"); assert!(s.chars().filter(|c| c.is_ascii_digit()).count() == 14); }
    #[test]
    fn recording_path_is_inside_aufnahmen_and_clean() {
        let p = recording_path(Path::new("/p"), "mein lied/x"); let n = p.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(p.parent().unwrap(), Path::new("/p/aufnahmen")); assert!(n.starts_with("mein_lied_x_") && n.ends_with(".wav"), "{n}");
    }

    fn demo_song() -> (std::path::PathBuf, std::path::PathBuf, crate::tracker::TrackerEd) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/music_demo");
        let data = vc_ide::find_data_dir(&root).expect("data/");
        let ed = crate::tracker::TrackerEd::open(&root, "music/tune.vcsong", &data).unwrap();
        (root, data, ed)
    }

    #[test]
    fn compare_finds_delay_of_a_recording_of_the_synth_itself() {
        let (_, _, ed) = demo_song(); let rate = 47983u32;
        let syn = render_synth(ed.song(false), rate as f32, 14.0);
        assert!(syn.iter().any(|v| v.abs() > 0.01), "der Synthesizer liefert Ton");
        // „Aufnahme“: 0,6 s Stille davor, halb so laut, als WAV geschrieben und wieder gelesen
        let path = std::env::temp_dir().join(format!("vc-cmp-{}.wav", std::process::id()));
        { let mut w = wav::WavWriter::create(&path, rate).unwrap();
          let pad = (rate as f32 * 0.6) as usize;
          w.write(std::iter::repeat((0i16, 0i16)).take(pad).chain(syn.iter().take(rate as usize * 8).map(|v| { let q = (v * 0.5 * 32767.0) as i16; (q, q) }))).unwrap(); }
        let c = build_compare(ed.song(false), &path).unwrap(); let _ = std::fs::remove_file(&path);
        let m = c.cmp.unwrap();
        assert!((m.lag_secs - 0.6).abs() < 0.03, "{m:?}"); assert!(m.corr > 0.9, "{m:?}"); assert!((m.diff_db + 6.0).abs() < 1.0, "{m:?}");
        // Spektren gleicher Form: Unterschied der Bandpegel klein (nur Lautstärke 6 dB = 0,1 der Skala)
        let d: f32 = (0..viz::BANDS).map(|b| (c.spec_a[b] - c.spec_b[b]).abs()).sum::<f32>() / viz::BANDS as f32; assert!(d < 0.15, "{d}");
        assert!(c.spec_a.iter().any(|v| *v > 0.2));
    }

    #[test]
    fn compare_rejects_short_and_broken_files() {
        let (_, _, ed) = demo_song();
        assert!(build_compare(ed.song(false), Path::new("/nicht/da.wav")).is_err());
        let path = std::env::temp_dir().join(format!("vc-short-{}.wav", std::process::id()));
        { let mut w = wav::WavWriter::create(&path, 47983).unwrap(); w.write(std::iter::repeat((1i16, 1i16)).take(100)).unwrap(); }
        assert!(build_compare(ed.song(false), &path).unwrap_err().contains("zu kurz")); let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn strip_and_compare_window_draw_in_both_themes() {
        let (root, data, ed) = demo_song();
        let audio = Audio::new(); let json = || vc_media::project::format_media(&serde_json::json!({})); let song = || ed.song(false);
        for light in [false, true] {
            crate::theme::set_light(light);
            let ctx = egui::Context::default(); crate::fonts::install(&ctx);
            let mut p = ListenPanel::default();
            p.cmp = Some(Compare { rate: 47983.0, file: "tune_20261108-130000.wav".into(), cmp: Some(wav::Comparison { lag_secs: 0.62, corr: 0.91, rms_a_db: -21.0, rms_b_db: -19.5, diff_db: 1.5, overlap_secs: 12.0 }),
                spec_a: [0.5; viz::BANDS], spec_b: [0.4; viz::BANDS], lv_a: (-21.0, -9.0), lv_b: (-19.5, -6.0) });
            for (view, confirm) in [("both", false), ("a", false), ("b", true)] {
                p.cmp_open = true; p.cmp_view = view.into(); p.confirm = confirm;
                p.machine = Some(("U64".into(), "192.168.0.10".into(), String::new())); p.machine_at = Some(std::time::Instant::now());
                let info = SongInfo { root: &root, data: &data, name: "tune", profile: "c64.sid.song", chip: "sid", channels: 3, hz: 50.0, json: &json, song: &song };
                let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(1100.0, 760.0))), ..Default::default() };
                let mut out = ctx.run_ui(input, |ui| { p.show(ui, &audio, &info); }); out.textures_delta.clear();
            }
        }
        crate::theme::set_light(false);
    }

    #[test]
    fn unsupported_song_is_not_selectable_and_gives_reason() {
        let (root, data, ed) = demo_song(); let audio = Audio::new(); let json = || String::new(); let song = || ed.song(false);
        let mut p = ListenPanel::default(); p.machine = Some(("U64".into(), "h".into(), String::new())); p.machine_at = Some(std::time::Instant::now());
        let info = SongInfo { root: &root, data: &data, name: "x", profile: "atari.pokey.song", chip: "pokey", channels: 4, hz: 50.0, json: &json, song: &song };
        let ctx = egui::Context::default(); crate::fonts::install(&ctx);
        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(1100.0, 400.0))), ..Default::default() };
        let mut out = ctx.run_ui(input, |ui| { p.show(ui, &audio, &info); }); out.textures_delta.clear();
        assert_eq!(audio.source(), Source::Synth); assert!(!p.running());
        assert!(vc_ide::listen::unsupported_reason(info.profile, info.chip, info.channels).unwrap().contains("POKEY"));
    }

    /// Mit einer echten Aufnahme: `VC_REC_WAV=aufnahme.wav cargo test -p vc-app real_recording -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_recording_against_synth() {
        let Ok(f) = std::env::var("VC_REC_WAV") else { return };
        let (_, _, ed) = demo_song(); let c = build_compare(ed.song(false), Path::new(&f)).unwrap();
        println!("{:?}\nA {:?}\nB {:?}\nspec A {:?}\nspec B {:?}", c.cmp, c.lv_a, c.lv_b, c.spec_a, c.spec_b);
    }
}
