// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder, neue Oberfläche (Tauri 2). Hülle um den Rust-Kern; die Oberfläche selbst liegt in `ui/` (Svelte, TypeScript).
//! Rust bleibt die Quelle der Wahrheit (Dokumente, Verlauf, Dateiformate); die Oberfläche zeigt und sendet Absichten.
#![recursion_limit = "512"] // die große Ansicht des Sprite-Editors (json!) braucht mehr Tiefe
pub mod ablegen;
pub mod markdown;
pub mod audio;
pub mod bau;
pub mod bildschirm;
pub mod bench;
pub mod cmds;
pub mod daten;
pub mod einst;
pub mod hilfe;
pub mod info;
pub mod wiz;
pub mod dbgsitzung;
pub mod debug;
pub mod dateien;
pub mod disk;
pub mod palette;
pub mod pixel;
pub mod projekt;
pub mod export;
pub mod docstore;
pub mod erweiterungen;
pub mod events;
pub mod fenster;
pub mod kacheln;
pub mod karten;
pub mod lsp;
pub mod menu;
pub mod neuprojekt;
pub mod sitzung;
pub mod startargs;
pub mod start;
pub mod suche;
pub mod testlauf;
pub mod tracker;
pub mod basic;
pub mod turbo;
pub mod text;
#[cfg(feature = "selftest")]
pub static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
#[cfg(feature = "testbridge")]
pub mod testbridge;

use serde_json::Value;
use tauri::{AppHandle, Manager};

/// Einziger Einstieg der Oberfläche in den Kern (Befehlsname und JSON-Argumente); Antworten und Fehler sind JSON
#[tauri::command]
async fn vc(app: AppHandle, cmd: String, args: Option<Value>) -> Result<Value, docstore::Fehler> {
    // Eine Panik in einem Befehl darf das Programm nicht beenden: sie wird zum sichtbaren Fehler `intern` (Meldung in der Oberfläche) und steht im Panikprotokoll
    let a = args.unwrap_or(Value::Null);
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cmds::dispatch(&app, &cmd, a)))
        .unwrap_or_else(|_| Err(docstore::Fehler::mit("intern", serde_json::json!({ "cmd": cmd }))))
}

/// Panikprotokoll: jede Panik landet mit Ort und Meldung in `vintagecoder-panik.log` im Temp-Ordner und auf stderr
fn panik_protokoll() {
    std::panic::set_hook(Box::new(|info| {
        let text = format!("{info}\n");
        eprintln!("PANIK: {text}");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(std::env::temp_dir().join("vintagecoder-panik.log")) { use std::io::Write; let _ = f.write_all(text.as_bytes()); }
    }));
}

/// Binärer Weg (Messung G1): `n` Bytes ohne JSON-Umweg; später Livebild, PNG-Blätter, Hex-Seiten
#[tauri::command]
fn vc_bin(n: usize) -> tauri::ipc::Response { tauri::ipc::Response::new(vec![0x5Au8; n.min(1 << 24)]) }

/// Gemerkte Größe und Lage anwenden (Fenster ist beim Start noch nicht gezeigt oder gerade erst)
#[cfg(not(any(feature = "testbridge", feature = "selftest")))]
fn fenster_wiederherstellen(app: &tauri::App) {
    let Some(w) = app.get_webview_window("main") else { return };
    let schirme: Vec<fenster::Schirm> = w.available_monitors().unwrap_or_default().iter().map(|m| fenster::Schirm { x: m.position().x, y: m.position().y, w: m.size().width, h: m.size().height, skala: m.scale_factor() }).collect();
    let haupt = w.primary_monitor().ok().flatten().map(|m| fenster::Schirm { x: m.position().x, y: m.position().y, w: m.size().width, h: m.size().height, skala: m.scale_factor() });
    let mut s = schirme; if let Some(h) = haupt { s.retain(|x| *x != h); s.insert(0, h); }
    let gemerkt = fenster::datei().and_then(|f| fenster::lesen(&f));
    // macOS: Bildschirme und Fenster liegen in logischen Punkten (gemischte Skalierung: Retina und normaler Schirm); dort wird in Punkten gerechnet und angewendet.
    // Sonst (Windows, Linux): physische Pixel; erst die Lage setzen (das Fenster wechselt auf den Zielschirm, ggf. mit anderer Skalierung), dann die Größe.
    #[cfg(target_os = "macos")]
    {
        let p = fenster::plan_logisch(gemerkt, &s);
        let _ = w.set_size(tauri::LogicalSize::new(p.groesse.0, p.groesse.1));
        match p.pos { Some((x, y)) => { let _ = w.set_position(tauri::LogicalPosition::new(x, y)); } None => { let _ = w.center(); } }
        if p.maximiert { let _ = w.maximize(); }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let p = fenster::plan(gemerkt, &s);
        match p.pos { Some((x, y)) => { let _ = w.set_position(tauri::PhysicalPosition::new(x, y)); } None => { let _ = w.center(); } }
        let _ = w.set_size(tauri::PhysicalSize::new(p.groesse.0, p.groesse.1));
        if p.maximiert { let _ = w.maximize(); }
    }
    // Während des Anwendens und der ersten Ereignisse beim Start (Resize/Move durch Anzeige und Skalierungswechsel) wird nichts gespeichert
    fenster_freigeben();
}

/// Erst nach dem Wiederherstellen und einer kurzen Beruhigung speichern Resize/Move-Ereignisse wieder (sonst überschreiben die Ereignisse beim Start den gemerkten Stand)
#[cfg(not(any(feature = "testbridge", feature = "selftest")))]
static FREI: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(not(any(feature = "testbridge", feature = "selftest")))]
fn fenster_freigeben() { std::thread::spawn(|| { std::thread::sleep(std::time::Duration::from_millis(2000)); FREI.store(true, std::sync::atomic::Ordering::SeqCst); }); }

/// Größe und Lage speichern: beim Verstellen entprellt (400 ms nach dem letzten Resize/Move, damit der Endwert nicht verloren geht), beim Schließen sofort
#[cfg(not(any(feature = "testbridge", feature = "selftest")))]
fn fenster_merken(win: &tauri::Window, ev: &tauri::WindowEvent) {
    use std::sync::atomic::{AtomicU64, Ordering}; use std::time::Duration;
    static GEN: AtomicU64 = AtomicU64::new(0);
    let schliessen = matches!(ev, tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed);
    if schliessen { fenster_speichern(win); return; }
    if !matches!(ev, tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_)) || !FREI.load(Ordering::SeqCst) { return; }
    let g = GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let w = win.clone();
    std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(400)); if GEN.load(Ordering::SeqCst) == g { fenster_speichern(&w); } });
}

#[cfg(not(any(feature = "testbridge", feature = "selftest")))]
fn fenster_speichern(win: &tauri::Window) {
    let Some(f) = fenster::datei() else { return };
    if win.is_minimized().unwrap_or(false) || win.is_fullscreen().unwrap_or(false) { return; }
    let max = win.is_maximized().unwrap_or(false);
    let alt = fenster::lesen(&f);
    // maximiert: nur das Kennzeichen merken, Größe und Lage des normalen Fensters bleiben erhalten
    let z = if max { alt.map(|a| fenster::Zustand { maximiert: true, ..a }) } else {
        match (win.outer_position(), win.inner_size()) { (Ok(p), Ok(s)) => Some(fenster::Zustand { x: p.x, y: p.y, w: s.width, h: s.height, maximiert: false, skala: win.scale_factor().unwrap_or(0.0) }), _ => None }
    };
    if let Some(z) = z { fenster::schreiben(&f, &z); }
}

/// Größe und Lage des Hauptfensters beim Beenden sichern (nicht in Prüfbrücke und Selbstmessung)
fn fenster_beim_ende(app: &tauri::AppHandle) {
    #[cfg(not(any(feature = "testbridge", feature = "selftest")))]
    if let Some(w) = app.get_webview_window("main") { fenster_speichern(&w.as_ref().window()); }
    #[cfg(any(feature = "testbridge", feature = "selftest"))]
    let _ = app;
}

/// Kennung des Webview-Datenspeichers aus dem Konfigurationsordner (stabil, je Ordner verschieden): 16 Byte aus zwei FNV-1a-Hashes
pub fn speicher_kennung(dir: &str) -> [u8; 16] {
    let fnv = |seed: u64| { let mut h = seed; for b in dir.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); } h };
    let (a, b) = (fnv(0xcbf29ce484222325), fnv(0x84222325cbf29ce4));
    let mut k = [0u8; 16]; k[..8].copy_from_slice(&a.to_le_bytes()); k[8..].copy_from_slice(&b.to_le_bytes()); k
}
#[cfg(target_os = "macos")]
fn macos_ab_14() -> bool {
    std::process::Command::new("sw_vers").arg("-productVersion").output().ok().and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|v| v.trim().split('.').next().and_then(|m| m.parse::<u32>().ok())).is_some_and(|m| m >= 14)
}
/// Hauptfenster aus der Konfiguration bauen; mit eigenem Konfigurationsordner (`VC_CONFIG_DIR`: Tests, Prüfpakete, portable Nutzung) hat es auch eigene Webview-Daten
/// (Startbild, gemerkter Bereich, Zoom …): macOS ab 14 über eine Datenspeicher-Kennung, Windows und Linux über `<Konfigurationsordner>/webview`
fn fenster_erzeugen(app: &tauri::App) -> tauri::Result<()> {
    let cfg = app.config().app.windows.first().cloned().expect("Fensterkonfiguration fehlt");
    #[allow(unused_mut)] let mut b = tauri::WebviewWindowBuilder::from_config(app.handle(), &cfg)?;
    if let Some(dir) = std::env::var_os("VC_CONFIG_DIR").filter(|d| !d.is_empty()) {
        #[cfg(target_os = "macos")]
        if macos_ab_14() { b = b.data_store_identifier(speicher_kennung(&dir.to_string_lossy())); }
        #[cfg(not(target_os = "macos"))]
        { b = b.data_directory(std::path::PathBuf::from(&dir).join("webview")); }
        let _ = &dir;
    }
    b.build()?; Ok(())
}

pub fn run() {
    #[cfg(feature = "selftest")]
    let _ = START.set(std::time::Instant::now());
    panik_protokoll();
    cmds::sprache_beim_start();
    let b = tauri::Builder::default();
    // Einzel-Instanz: ein zweiter Aufruf (Kommandozeile, Dateizuordnung unter Windows/Linux) übergibt seine Pfade an die laufende App und holt sie nach vorn; nicht in der Prüfbrücke und der Selbstmessung (dort starten mehrere Kerne nacheinander)
    #[cfg(all(desktop, not(any(feature = "testbridge", feature = "selftest"))))]
    let b = b.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
        startargs::vormerken(startargs::aus_argumenten(args.into_iter().skip(1)));
        events::emit(app, "app:oeffnen", serde_json::json!({}));
        if let Some(w) = app.get_webview_window("main") { let _ = w.unminimize(); let _ = w.show(); let _ = w.set_focus(); }
    }));
    b
        .plugin(tauri_plugin_dialog::init())
        .manage(cmds::Shell::default())
        .invoke_handler(tauri::generate_handler![vc, vc_bin])
        .on_menu_event(|app, ev| {
            let id = ev.id().as_ref();
            events::emit(app, "menu:cmd", serde_json::json!({ "id": id }));
        })
        .on_window_event(|win, ev| {
            // Fenstergröße und -lage merken (nicht in der Prüfbrücke und der Selbstmessung: dort ist das Fenster versteckt oder verschoben)
            #[cfg(not(any(feature = "testbridge", feature = "selftest")))]
            if win.label() == "main" { fenster_merken(win, ev); }
            #[cfg(any(feature = "testbridge", feature = "selftest"))]
            { let _ = (win, ev); }
        })
        .setup(|app| {
            let h = app.handle().clone();
            fenster_erzeugen(app)?;   // das Hauptfenster entsteht hier (nicht aus der Konfiguration), damit seine Webview-Daten zum Konfigurationsordner gehören
            // Release-Auswahl (data/release.json) und Befehle aus der Registry, bevor Menü und Palette entstehen
            if let Some(w) = vc_ide::features::init(lsp::daten_ordner().as_deref()) { eprintln!("Release: {w}"); }
            for m in erweiterungen::befehle_laden() { eprintln!("Erweiterungen: {m}"); }
            // erst die Menüleiste, dann die gemerkte Größe: unter Windows nimmt die Menüleiste Höhe vom Innenbereich, sonst schrumpfte das Fenster bei jedem Start
            if let Err(e) = cmds::menue_neu(&h) { eprintln!("Menü: {}", e.code); }
            #[cfg(not(any(feature = "testbridge", feature = "selftest")))]
            fenster_wiederherstellen(app);
            #[cfg(feature = "selftest")]
            {
                // Selbstmessung: Fenster außerhalb des Bildschirms, kein Dock-Symbol
                #[cfg(target_os = "macos")]
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                if std::env::var_os("VC_SELFTEST_OUT").is_some() { if let Some(w) = app.get_webview_window("main") { let _ = w.set_ignore_cursor_events(true); let _ = w.set_position(tauri::PhysicalPosition::new(-30000, -30000)); } }   // weit außerhalb des Bildschirms (ein verborgenes Fenster führt keine Skripte aus)
            }
            #[cfg(feature = "testbridge")]
            {
                // Prüfbrücke: der Kern wird nur über WebSocket gebraucht; kein sichtbares Fenster, kein Dock-Symbol, kein Fokus (stört niemanden beim Testen am selben Rechner)
                #[cfg(target_os = "macos")]
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                if let Some(w) = app.get_webview_window("main") { let _ = w.hide(); }
                testbridge::start(h.clone());
            }
            let _ = app.state::<cmds::Shell>();
            Ok(())
        })
        .build({ let a = startargs::aus_argumenten(std::env::args().skip(1)); startargs::vormerken(a); tauri::generate_context!() })
        .expect("Fehler beim Start von VintageCoder")
        // beim Beenden der IDE geht ein per F5 gestarteter Emulator mit (samt allen Wrapper-Prozessen)
        .run(|app, ev| match ev {
            // Cmd+Q und „Beenden“ gehen über `app.exit` und nicht über das Schließen des Fensters: Größe und Lage hier synchron schreiben (ExitRequested und Exit, je nach Weg)
            tauri::RunEvent::ExitRequested { .. } => { fenster_beim_ende(app); }
            tauri::RunEvent::Exit => { fenster_beim_ende(app); vc_ide::launch::stop_running(); }
            // macOS: „Öffnen mit“, Doppelklick auf eine Datei, `open -a VintageCoder <pfad>`: die Pfade merken und die Oberfläche benachrichtigen
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Opened { urls } => {
                let pfade: Vec<String> = urls.iter().filter_map(|u| u.to_file_path().ok()).map(|p| p.to_string_lossy().into_owned()).collect();
                if !pfade.is_empty() { startargs::vormerken(startargs::Auftrag { pfade, ..Default::default() }); events::emit(app, "app:oeffnen", serde_json::json!({})); }
            }
            _ => {}
        });
}

#[cfg(test)]
mod tests_webview {
    use super::speicher_kennung;
    #[test] fn kennung_je_konfigurationsordner_stabil_und_verschieden() {
        assert_eq!(speicher_kennung("/tmp/a/config"), speicher_kennung("/tmp/a/config"));
        assert_ne!(speicher_kennung("/tmp/a/config"), speicher_kennung("/tmp/b/config"));
        assert_ne!(speicher_kennung("/tmp/a/config")[..8], speicher_kennung("/tmp/a/config")[8..], "die beiden Hälften sind verschieden gehasht");
    }
}
