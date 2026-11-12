// SPDX-License-Identifier: GPL-3.0-or-later
//! Reiter „Live-Bild“ (Debugger): Bild der Ultimate 64 aus dem UDP-Bildstrom, Statuschip, Speichern und Zwischenablage.
//!
//! Ein Hintergrundthread besitzt den [`StreamService`] (Start und Stopp machen REST-Aufrufe), setzt die Pakete zu Bildern zusammen und legt
//! nur das neueste Bild ab; die Oberfläche lädt es im Zeichenthread als Textur. Der Strom startet nur auf ausdrückliche Handlung
//! (`request_start`) und endet beim Schließen des Reiters, bei „Alle Ströme stoppen“ und beim Beenden (Drop).
use crate::ui::tokens::{t, tint};
use crate::ui::widgets;
use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vc_emu::streams::video::Frame;
use vc_emu::streams::StreamKind;
use vc_ide::livebild::VideoFeed;
use vc_ide::streams::{StreamOptions, StreamService, StreamState, StreamStatus};

/// Ultimate, auf der das Live-Bild läuft
#[derive(Clone, Debug, PartialEq)]
pub struct MachineInfo { pub name: String, pub host: String, pub password: String }

/// Was die Anwendung dem Reiter mitgibt
pub struct Env { pub machine: Option<MachineInfo>, pub opts: StreamOptions, pub root: Option<PathBuf> }

enum Cmd { Start { host: String, password: String, opts: StreamOptions }, StopAll, Quit }

#[derive(Default)]
struct Shared { frame: Option<Arc<Frame>>, seq: u64, status: Option<StreamStatus>, fps: f64, busy: bool }

enum Act { Start, Save, Copy, StopAll }

pub struct LiveState {
    shared: Arc<Mutex<Shared>>,
    tx: Option<Sender<Cmd>>,
    thread: Option<std::thread::JoinHandle<()>>,
    tex: Option<egui::TextureHandle>,
    tex_seq: u64,
    frame: Option<Arc<Frame>>,
    want_start: bool,
    /// Erklärung zur Systemabfrage schon bestätigt?
    hint_seen: bool,
    msg: Option<(bool, String)>,
    /// Selbsttest: feste Beispielwerte statt eines Geräts
    demo: Option<String>,
}

impl Default for LiveState { fn default() -> Self { LiveState::new() } }

impl LiveState {
    pub fn new() -> LiveState {
        LiveState { shared: Default::default(), tx: None, thread: None, tex: None, tex_seq: 0, frame: None, want_start: false, hint_seen: hint_seen(), msg: None, demo: None }
    }

    /// Ausdrückliche Handlung des Nutzers: Strom starten (nach der einmaligen Erklärung)
    pub fn request_start(&mut self) { self.want_start = true; }

    /// Läuft oder startet gerade ein Strom?
    pub fn active(&self) -> bool {
        self.demo.is_some() || self.shared.lock().ok().and_then(|s| s.status.as_ref().map(|st| matches!(st.state, StreamState::Starting | StreamState::Running | StreamState::Lossy))).unwrap_or(false)
    }

    /// Strom beenden (Schließen des Reiters)
    pub fn stop(&mut self) {
        self.want_start = false;
        if let Some(tx) = &self.tx { let _ = tx.send(Cmd::StopAll); }
    }

    fn ensure_thread(&mut self, ctx: &egui::Context) -> &Sender<Cmd> {
        if self.tx.is_none() {
            let (tx, rx) = channel();
            let (sh, ctx) = (self.shared.clone(), ctx.clone());
            self.thread = std::thread::Builder::new().name("live-bild".into()).spawn(move || worker(rx, sh, ctx)).ok();
            self.tx = Some(tx);
        }
        self.tx.as_ref().expect("Thread")
    }

    /// Selbsttest (`VC_DEMO=live`): Bild aus aufgezeichneten Paketen und feste Beispielwerte, kein Gerät. `err`: `lan` oder `fw` zeigt die Fehlerkarte.
    pub fn demo(&mut self, bin: &std::path::Path, err: Option<&str>) {
        let mut asm = vc_emu::streams::video::FrameAssembler::new();
        let mut frame = None;
        if let Ok(b) = std::fs::read(bin) {
            let mut o = 0;
            while o + 2 <= b.len() { let n = u16::from_le_bytes([b[o], b[o + 1]]) as usize; if o + 2 + n > b.len() { break; } if let Ok(Some(f)) = asm.push(&b[o + 2..o + 2 + n]) { frame = Some(f); } o += 2 + n; }
        }
        let target = Some(std::net::SocketAddrV4::new([192, 168, 22, 66].into(), 51342));
        let state = match err {
            Some("lan") => StreamState::Error("Die Ultimate meldet: No Operational Network Interface".into()),
            Some("fw") => StreamState::Error(vc_ide::streams::no_packets_hint(StreamKind::Video, target, false)),
            Some("loss") => StreamState::Lossy,
            _ => StreamState::Running,
        };
        let loss = if err == Some("loss") { 6.2 } else { 0.0 };
        let mut sh = self.shared.lock().unwrap();
        sh.frame = frame.map(Arc::new); sh.seq = 1; sh.fps = 50.0;
        sh.status = Some(StreamStatus { kind: StreamKind::Video, state, target, packets: 170_000, lost: 0, rejected: 0, loss_percent: loss, packets_per_s: 3400.0, mbit_per_s: 21.3, since_last: Some(Duration::from_millis(3)), restarts: 0 });
        drop(sh);
        self.demo = Some("192.168.0.10".into()); self.hint_seen = true;
    }

    fn pull_texture(&mut self, ctx: &egui::Context) {
        let (frame, seq) = match self.shared.lock() { Ok(s) => (s.frame.clone(), s.seq), Err(_) => return };
        if seq != self.tex_seq {
            self.tex_seq = seq;
            if let Some(f) = frame {
                let img = egui::ColorImage::from_rgba_unmultiplied([f.width, f.height], &f.rgba);
                match &mut self.tex { Some(t) => t.set(img, egui::TextureOptions::NEAREST), None => self.tex = Some(ctx.load_texture("live-bild", img, egui::TextureOptions::NEAREST)) }
                self.frame = Some(f);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        let k = t();
        let ctx = ui.ctx().clone();
        self.pull_texture(&ctx);
        let (status, fps, busy) = match self.shared.lock() { Ok(s) => (s.status.clone(), s.fps, s.busy), Err(_) => (None, 0.0, false) };
        let running = status.as_ref().map_or(false, |s| matches!(s.state, StreamState::Starting | StreamState::Running | StreamState::Lossy));
        let mut act: Option<Act> = None;

        // Start auf ausdrückliche Handlung, nach der einmaligen Erklärung
        let machine = env.machine.clone().or_else(|| self.demo.as_ref().map(|h| MachineInfo { name: "Ultimate 64".into(), host: h.clone(), password: String::new() }));
        if self.want_start && machine.is_some() && self.hint_seen && !busy && !running { self.want_start = false; act = Some(Act::Start); }
        if machine.is_none() { self.want_start = false; }

        // Kopfzeile: Titel, Umschalter, Statuschip
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Live-Bild").font(egui::FontId::new(15.0, crate::fonts::ui_strong())));
            if let Some(m) = &machine { ui.label(RichText::new(format!("· {}", m.name)).color(k.dim)); }
            self.mode_switch(ui);
            if let Some(s) = &status { status_chip(ui, s, fps, busy); } else if busy { widgets::chip(ui, "startet …", k.am); } else { widgets::chip(ui, "aus", k.dim); }
            if self.hint_seen { ui.separator(); self.buttons(ui, running, &mut act); }
        });
        ui.add_space(4.0);

        let Some(machine) = machine else {
            widgets::card(ui, false, |ui| {
                ui.label(RichText::new("Keine Ultimate 64 eingerichtet").strong());
                ui.label(RichText::new("Das Live-Bild zeigt den Bildschirm einer Ultimate 64 über das Netzwerk. Trage in den Einstellungen unter „Maschinen“ eine Ultimate 64 ein.").color(k.mute));
            });
            return;
        };

        if !self.hint_seen && !running {
            self.hint_card(ui);
            return;
        }

        // Inhalt: Bild links, Seitenleiste rechts (bei schmalem Reiter nur die Knöpfe oben)
        let wide = ui.available_width() > 640.0;
        let area = ui.available_rect_before_wrap();
        let side_w = 270.0;
        let (img_area, side_area) = if wide { (egui::Rect::from_min_max(area.min, egui::pos2(area.right() - side_w - 12.0, area.bottom())), Some(egui::Rect::from_min_max(egui::pos2(area.right() - side_w, area.top()), area.max))) } else { (area, None) };
        let img_area = if wide { img_area } else { ui.available_rect_before_wrap() };
        self.image_area(ui, img_area, status.as_ref(), running || busy, &mut act);
        if let Some(sa) = side_area {
            ui.scope_builder(egui::UiBuilder::new().max_rect(sa), |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| { self.side(ui, &machine, status.as_ref(), fps, running); });
            });
        }
        if !wide { ui.allocate_rect(img_area, egui::Sense::hover()); } else { ui.allocate_rect(area, egui::Sense::hover()); }

        // Aktionen
        match act {
            Some(Act::Start) => {
                self.msg = None;
                if let Ok(mut s) = self.shared.lock() { s.busy = true; }
                let (host, password, opts) = (machine.host.clone(), machine.password.clone(), env.opts.clone());
                let tx = self.ensure_thread(&ctx).clone();
                let _ = tx.send(Cmd::Start { host, password, opts });
            }
            Some(Act::StopAll) => { self.msg = None; self.stop(); if self.demo.is_some() { self.demo = None; } }
            Some(Act::Save) => self.msg = Some(self.save(env)),
            Some(Act::Copy) => self.msg = Some(self.copy(&ctx)),
            None => {}
        }
        if running || busy { ctx.request_repaint_after(Duration::from_millis(100)); }
    }

    fn mode_switch(&self, ui: &mut egui::Ui) {
        let k = t();
        egui::Frame::new().stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(1)).show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            ui.horizontal(|ui| {
                // „Bild“ ist die einzige wählbare Ansicht bis zum Debug-Strom
                ui.add(egui::Button::new(RichText::new("Bild").strong().color(k.cy)).fill(tint(k.cy)).stroke(egui::Stroke::NONE).corner_radius(3.0));
                let r = ui.add_enabled(false, egui::Button::new(RichText::new("Ablaufprotokoll").color(k.dim)).fill(Color32::TRANSPARENT).stroke(egui::Stroke::NONE).corner_radius(3.0));
                r.on_disabled_hover_text("Das Ablaufprotokoll (Debug-Strom, Buszyklen) folgt in einer späteren Ausbaustufe. Bild und Ablaufprotokoll laufen nie zugleich: Der Umschalter stoppt jeweils den anderen Strom.");
            });
        });
    }

    fn hint_card(&mut self, ui: &mut egui::Ui) {
        let k = t();
        widgets::card(ui, false, |ui| {
            ui.label(RichText::new("Bevor das Live-Bild startet").strong());
            ui.add_space(2.0);
            ui.label(RichText::new("Die Ultimate schickt das Bild als UDP-Pakete an diesen Rechner. Dafür öffnet VintageCoder einen Empfangsanschluss im lokalen Netz. Das System fragt deshalb beim ersten Mal, ob das Programm „eingehende Verbindungen“ annehmen darf. Bitte erlaube es; sonst kommen keine Pakete an.").color(k.mute));
            ui.add_space(2.0);
            ui.label(RichText::new("Es wird nichts in der Ultimate gespeichert und keine ihrer Einstellungen geändert. Beim Schließen des Reiters stoppt VintageCoder den Strom wieder.").color(k.mute));
            ui.add_space(6.0);
            if crate::shell::action_button(ui, "Verstanden, Live-Bild starten", crate::shell::Kind::Primary, true).clicked() { set_hint_seen(); self.hint_seen = true; }
        });
    }

    fn buttons(&mut self, ui: &mut egui::Ui, running: bool, act: &mut Option<Act>) {
        let has = self.frame.is_some();
        if !running && crate::shell::action_button(ui, "Live-Bild starten", crate::shell::Kind::Primary, self.demo.is_none()).clicked() { self.want_start = true; }
        if ui.add_enabled(has, egui::Button::new("Live-Bild speichern")).on_hover_text("PNG im Projekt unter aufnahmen/").clicked() { *act = Some(Act::Save); }
        if ui.add_enabled(has, egui::Button::new("In Zwischenablage")).clicked() { *act = Some(Act::Copy); }
        if ui.add_enabled(running, egui::Button::new("Alle Ströme stoppen")).clicked() { *act = Some(Act::StopAll); }
    }

    fn image_area(&mut self, ui: &mut egui::Ui, rect: egui::Rect, status: Option<&StreamStatus>, live: bool, act: &mut Option<Act>) {
        let k = t();
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 4.0, k.bg0);
        p.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside);
        let inner = rect.shrink(8.0);
        if let Some(StreamState::Error(e)) = status.map(|s| &s.state) {
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| { egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| { if error_card(ui, e) { self.want_start = true; let _ = act; } }); });
            return;
        }
        if let (Some(tex), Some(f)) = (&self.tex, &self.frame) {
            let (w, h) = (f.width as f32, f.height as f32);
            if inner.width() > 8.0 && inner.height() > 8.0 {
                let scale = (inner.width() / w).min(inner.height() / h);
                let size = egui::vec2(w * scale, h * scale);
                let r = egui::Rect::from_center_size(inner.center(), size);
                p.image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
            }
            if !live { p.text(rect.left_top() + egui::vec2(14.0, 12.0), egui::Align2::LEFT_TOP, "angehalten (letztes Bild)", egui::FontId::proportional(12.0), k.am); }
        } else {
            let txt = if live { "Warte auf das erste Bild …" } else { "Das Live-Bild ist aus." };
            p.text(rect.center(), egui::Align2::CENTER_CENTER, txt, egui::FontId::proportional(14.0), k.dim);
        }
    }

    fn side(&mut self, ui: &mut egui::Ui, m: &MachineInfo, status: Option<&StreamStatus>, fps: f64, running: bool) {
        let k = t();
        let row = |ui: &mut egui::Ui, l: &str, v: &str| { ui.horizontal_wrapped(|ui| { ui.label(RichText::new(l).color(k.dim)); ui.label(RichText::new(v).monospace().size(12.0)); }); };
        widgets::card(ui, false, |ui| {
            ui.label(RichText::new("Verbindung").strong());
            row(ui, "Gerät", &m.host);
            if let Some(tg) = status.and_then(|s| s.target) { row(ui, "Empfänger", &format!("{tg} (UDP)")); }
            if let Some(s) = status.filter(|s| s.packets > 0) { row(ui, "Verluste", &format!("{} % · {} Mbit/s", de(s.loss_percent, 1), de(s.mbit_per_s, 1))); row(ui, "Bilder", &format!("{} je Sekunde", de(fps, 0))); }
            if let Some(s) = status.filter(|s| s.restarts > 0) { row(ui, "Neustarts", &s.restarts.to_string()); }
        });
        ui.add_space(6.0);
        widgets::card(ui, false, |ui| {
            ui.label(RichText::new("Ströme").strong());
            ui.horizontal(|ui| { if running { widgets::chip(ui, "Läuft", k.gr); } else { widgets::chip(ui, "Aus", k.dim); } ui.label("Video"); });
            ui.horizontal(|ui| { widgets::chip(ui, "Aus", k.dim); ui.label(RichText::new("Audio").color(k.mute)); ui.label(RichText::new("(im Tracker)").small().color(k.dim)); });
            ui.horizontal_wrapped(|ui| { widgets::chip(ui, "Aus", k.dim); ui.label(RichText::new("Debug (6510-Bus)").color(k.mute)); ui.label(RichText::new("nicht zugleich mit Bild").small().color(k.dim)); });
        });
        ui.add_space(6.0);
        if let Some((ok, text)) = &self.msg { ui.label(RichText::new(text).small().color(if *ok { k.gr } else { k.rd })); }
        let (dot, txt) = if running { (k.gr, "Ultimate verbunden") } else { (k.dim, "Nicht verbunden") };
        ui.horizontal(|ui| { ui.label(RichText::new("●").color(dot)); ui.label(RichText::new(txt).color(k.mute)); });
        if let Some(f) = &self.frame { ui.label(RichText::new(format!("Live-Bild: {} × {}", f.width, f.height)).small().color(k.dim)); }
        ui.label(RichText::new("Beim Schließen werden die Ströme gestoppt").small().color(k.dim));
    }

    fn save(&self, env: &Env) -> (bool, String) {
        let Some(f) = &self.frame else { return (false, "Noch kein Bild empfangen.".into()) };
        let Some(root) = &env.root else { return (false, "Zum Speichern zuerst ein Projekt öffnen.".into()) };
        let path = vc_ide::livebild::shot_path(root);
        match vc_ide::livebild::save_png(f, &path) {
            Ok(()) => (true, format!("Gespeichert: {}", path.strip_prefix(root).unwrap_or(&path).display())),
            Err(e) => (false, format!("Speichern fehlgeschlagen: {e}")),
        }
    }

    fn copy(&self, ctx: &egui::Context) -> (bool, String) {
        let Some(f) = &self.frame else { return (false, "Noch kein Bild empfangen.".into()) };
        ctx.copy_image(egui::ColorImage::from_rgba_unmultiplied([f.width, f.height], &f.rgba));
        (true, "Bild in die Zwischenablage kopiert.".into())
    }
}

impl Drop for LiveState {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() { let _ = tx.send(Cmd::Quit); }
        if let Some(h) = self.thread.take() { let _ = h.join(); }
    }
}

/// Zahl mit Komma als Dezimalzeichen
pub fn de(v: f64, digits: usize) -> String { format!("{v:.digits$}").replace('.', ",") }

/// Statuschip: Rate und Verluste; gelb ab 1 % Verlust, rot ab 5 %
fn status_chip(ui: &mut egui::Ui, s: &StreamStatus, fps: f64, busy: bool) {
    let k = t();
    let (text, col) = match &s.state {
        StreamState::Off => if busy { ("startet …".to_string(), k.am) } else { ("aus".to_string(), k.dim) },
        StreamState::Starting => ("startet … wartet auf das erste Bild".to_string(), k.am),
        StreamState::Running | StreamState::Lossy => {
            let col = if s.loss_percent >= 5.0 { k.rd } else if s.loss_percent >= 1.0 { k.am } else { k.gr };
            (format!("{} Bilder/s · {} Mbit/s · {} % Verluste", de(fps, 0), de(s.mbit_per_s, 1), de(s.loss_percent, 1)), col)
        }
        StreamState::Error(_) => ("Fehler".to_string(), k.rd),
    };
    widgets::chip(ui, &text, col);
}

/// Überschrift und Erklärung zu einem Fehlertext des Dienstes (Mockup 3)
pub fn explain(err: &str) -> (&'static str, String) {
    if err.contains("No Operational Network Interface") || err.contains("Network Interface") {
        ("Die Ultimate hat kein LAN-Kabel angesteckt.", "Bild, Ton und Bus-Protokoll werden nur über den LAN-Anschluss gesendet, nicht über das WLAN der Ultimate. Das Gerät meldet „No Operational Network Interface“.".into())
    } else if err.contains("keine Pakete") || err.contains("Strom ausgefallen") {
        ("Es kommen keine Bildpakete an.", err.to_string())
    } else if err.contains("nicht erreichbar") {
        ("Die Ultimate ist nicht erreichbar.", err.to_string())
    } else {
        ("Der Bildstrom konnte nicht gestartet werden.", err.to_string())
    }
}

/// Fehlerkarte; liefert `true` bei „Erneut versuchen“
fn error_card(ui: &mut egui::Ui, err: &str) -> bool {
    let k = t();
    let (head, body) = explain(err);
    let mut retry = false;
    ui.horizontal(|ui| { widgets::chip(ui, "Fehler", k.rd); ui.label(RichText::new("Ströme der Ultimate nicht verfügbar").strong()); });
    ui.add_space(4.0);
    ui.label(RichText::new(head).color(k.fg).strong());
    ui.label(RichText::new(body).color(k.mute));
    ui.add_space(4.0);
    ui.label(RichText::new("Weiter nutzbar: Speicher lesen und schreiben, Programme starten, Hardware-Tests im Postfach-Modus.").small().color(k.dim));
    ui.add_space(6.0);
    if ui.button("Erneut versuchen").clicked() { retry = true; }
    retry
}

fn worker(rx: std::sync::mpsc::Receiver<Cmd>, sh: Arc<Mutex<Shared>>, ctx: egui::Context) {
    use std::sync::mpsc::RecvTimeoutError;
    let (mut svc, mut feed): (Option<StreamService>, Option<VideoFeed>) = (None, None);
    let (mut last_poll, mut fps_t, mut fps_n) = (Instant::now(), Instant::now(), 0u32);
    loop {
        match rx.recv_timeout(Duration::from_millis(15)) {
            Ok(Cmd::Start { host, password, opts }) => {
                drop(svc.take()); drop(feed.take());   // alter Strom endet zuerst
                let mut s = StreamService::new(&host, Some(&password), opts);
                let (f, sink) = VideoFeed::new();
                let _ = s.start(StreamKind::Video, sink);          // Fehler stehen danach im Zustand des Stroms
                feed = Some(f); fps_t = Instant::now(); fps_n = 0;
                if let Ok(mut g) = sh.lock() { g.status = Some(s.status(StreamKind::Video)); g.busy = false; g.fps = 0.0; }
                svc = Some(s); ctx.request_repaint();
            }
            Ok(Cmd::StopAll) => {
                if let Some(s) = svc.as_mut() { s.stop_all(); if let Ok(mut g) = sh.lock() { g.status = Some(s.status(StreamKind::Video)); g.fps = 0.0; g.busy = false; } }
                feed = None; ctx.request_repaint();
            }
            Ok(Cmd::Quit) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
        let Some(s) = svc.as_mut() else { continue };
        if let Some(f) = feed.as_mut() {
            if let Some(frame) = f.pump(20_000) {
                fps_n += 1;
                if let Ok(mut g) = sh.lock() { g.frame = Some(Arc::new(frame)); g.seq += 1; }
                ctx.request_repaint();
            }
        }
        if last_poll.elapsed() >= Duration::from_millis(100) {
            last_poll = Instant::now();
            s.poll();
            let dt = fps_t.elapsed().as_secs_f64();
            let fps = if dt >= 1.0 { let v = fps_n as f64 / dt; fps_t = Instant::now(); fps_n = 0; Some(v) } else { None };
            if let Ok(mut g) = sh.lock() {
                g.status = Some(s.status(StreamKind::Video));
                if let Some(v) = fps { g.fps = v; }
                if matches!(g.status.as_ref().map(|x| &x.state), Some(StreamState::Off | StreamState::Error(_))) { g.fps = 0.0; }
            }
            ctx.request_repaint();
        }
    }
    drop(svc);   // stoppt die Ströme
}

fn prefs_file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("prefs.json")) }

fn hint_seen() -> bool {
    if std::env::var("VC_SHOT").is_ok() { return false; }
    prefs_file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).and_then(|v| v["live_hint"].as_bool()).unwrap_or(false)
}

fn set_hint_seen() {
    if std::env::var("VC_SHOT").is_ok() { return; }
    let Some(f) = prefs_file() else { return };
    let mut v = std::fs::read_to_string(&f).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).filter(|v| v.is_object()).unwrap_or_else(|| serde_json::json!({}));
    v["live_hint"] = serde_json::json!(true);
    if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); }
    let _ = std::fs::write(f, format!("{v}\n"));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn german_numbers() { assert_eq!(de(21.34, 1), "21,3"); assert_eq!(de(0.0, 1), "0,0"); assert_eq!(de(49.6, 0), "50"); }
    #[test]
    fn errors_are_explained() {
        assert!(explain("Die Ultimate meldet: No Operational Network Interface").0.contains("LAN-Kabel"));
        assert!(explain(&vc_ide::streams::no_packets_hint(StreamKind::Video, None, false)).0.contains("keine Bildpakete"));
        assert!(explain("Die Ultimate ist nicht erreichbar: timeout").0.contains("nicht erreichbar"));
        assert!(explain("sonst etwas").0.contains("nicht gestartet"));
    }
    #[test]
    fn demo_loads_real_frame_and_does_not_start() {
        let mut s = LiveState::new();
        s.demo(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../vc-emu/tests/data/video.bin"), None);
        assert!(s.active());
        assert_eq!(s.shared.lock().unwrap().frame.as_ref().map(|f| (f.width, f.height)), Some((384, 272)));
        assert!(s.tx.is_none(), "der Selbsttest startet keinen Thread");
    }
    #[test]
    fn start_needs_explicit_request() {
        let s = LiveState::new();
        assert!(!s.want_start && !s.active() && s.tx.is_none());
    }

    /// Zeichentest: Bild, Verluste, beide Fehlerkarten, ohne Maschine und vor der Erklärung, in schmal und breit
    #[test]
    fn live_bild_zeichnet_in_allen_zustaenden() {
        use eframe::egui::{pos2, vec2, Rect};
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        let bin = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../vc-emu/tests/data/video.bin");
        let machine = Some(MachineInfo { name: "Ultimate 64".into(), host: "192.168.0.10".into(), password: String::new() });
        for (w, h) in [(330.0, 300.0), (560.0, 380.0), (1000.0, 420.0)] {
            for variant in [None, Some("loss"), Some("lan"), Some("fw"), Some("keine"), Some("hinweis")] {
                let mut s = LiveState::new();
                match variant { Some("keine") | Some("hinweis") => s.hint_seen = false, Some(v) => s.demo(&bin, Some(v)), None => s.demo(&bin, None) }
                let env = Env { machine: if variant == Some("keine") { None } else { machine.clone() }, opts: StreamOptions::default(), root: None };
                for _ in 0..3 {
                    let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| s.ui(ui, &env));
                    o.textures_delta.clear();
                }
                assert!(s.tx.is_none(), "Zeichnen startet nie einen Strom ({variant:?})");
                if matches!(variant, None | Some("loss")) { assert!(s.tex.is_some() && s.frame.is_some(), "Bild als Textur ({variant:?})"); }
            }
        }
    }
}
