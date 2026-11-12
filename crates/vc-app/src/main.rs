// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder Native – egui-Oberfläche (Machbarkeitsprobe: Dock-Layout, Editor, Projektbaum, Bauen).
#![allow(deprecated)]
// Windows: GUI-Anwendung ohne mitstartendes Konsolenfenster
#![cfg_attr(windows, windows_subsystem = "windows")]
mod about;
mod feedback;
mod splash;
mod layout;
mod session;
mod editor_find;
mod app;
mod audio;
mod bitmap;
mod debug;
mod palette;
mod screen;
mod screenbank;
mod tiles;
mod snapshot;
mod editor;
mod dropinc;
mod fold;
mod pretty;
mod highlight;
mod theme;
mod charpick;
mod cmdpalette;
mod disk_studio;
mod debugger;
mod viz;
mod live;
mod projectpic;
mod emulator;
mod listen_ui;
mod synth;
mod tracker;
mod windows;
mod studios;
mod winctl;
mod lsp;
mod markdown;
mod help_images;
mod memmap;
mod overview;
mod project;
mod projsearch;
mod exportdlg;
mod projed;
mod fonts;
mod shell;
mod importdlg;
mod reorgdlg;
mod recent;
mod welcome;
mod wizard;
mod ui;
mod tests_panel;
mod settings_ui;
mod setup_wizard;
mod nativemenu;
mod rom_ui;

/// Linux: Fenstersymbol für Taskleiste und Fensterwechsel (X11 und Wayland); `app_id` `vintagecoder` passt zu `installer/linux/vintagecoder.desktop`.
/// macOS und Windows bekommen das Symbol aus dem Programmpaket bzw. der .exe.
fn window_icon(b: eframe::egui::ViewportBuilder) -> eframe::egui::ViewportBuilder {
    #[cfg(target_os = "linux")]
    {
        static PNG: &[u8] = include_bytes!("../../../installer/macos/VintageCoder-1024.png");
        if let Ok(img) = image::load_from_memory_with_format(PNG, image::ImageFormat::Png) {
            let img = img.resize(256, 256, image::imageops::FilterType::Triangle).to_rgba8();
            let (width, height) = img.dimensions();
            return b.with_icon(eframe::egui::IconData { rgba: img.into_raw(), width, height });
        }
    }
    b
}

/// Hilfe auf der Kommandozeile (`-h`, `--help`, `-V`, `--version`); `true`: Programm beenden
fn cli_help() -> bool {
    match std::env::args().nth(1).as_deref() {
        Some("-h") | Some("--help") => {
            println!("vintagecoder {} - die grafische Entwicklungsumgebung für Retro-Systeme\n", env!("CARGO_PKG_VERSION"));
            println!("Aufruf:  vintagecoder [projekt-ordner | datei.vcprj | datei.asm | datei.bas]\n");
            println!("Ohne Argument öffnet die IDE den Startbildschirm; mit einem Pfad öffnet sie das Projekt oder die Datei.\n");
            println!("Optionen\n  -h, --help       diese Hilfe\n  -V, --version    Version anzeigen\n");
            println!("Umgebung\n  VC_DATA          Ordner der Hardware-Datenbank (data)\n  VC_ROMS          ROM-Ordner des Benutzers\n  VC_XEMU          Pfad zum Xemu-Programm\n  VC_XEMU_SD       Ordner mit MEGA65.ROM für Xemu\n  VC_WIN           Fenstergröße, z. B. 1400x860\n");
            println!("Beispiele\n  vintagecoder                      # Startbildschirm\n  vintagecoder spiel                # Projekt im Ordner spiel öffnen\n  vintagecoder prog.asm             # Quelltext öffnen\n");
            println!("Kommandozeilenwerkzeug: vcc --help");
            true
        }
        Some("-V") | Some("--version") => { println!("vintagecoder {}", env!("CARGO_PKG_VERSION")); true }
        _ => false,
    }
}

fn main() -> eframe::Result {
    if cli_help() { return Ok(()); }
    vc_ide::debuglog::install_panic_hook();
    vc_ide::debuglog::init(vc_ide::settings::get().debug_log);
    let arg = std::env::args().nth(1).map(std::path::PathBuf::from);
    let opts = eframe::NativeOptions { viewport: window_icon(ui::titlebar::decorate(eframe::egui::ViewportBuilder::default().with_inner_size(std::env::var("VC_WIN").ok().and_then(|v| v.split_once('x').and_then(|(w, h)| Some([w.parse::<f32>().ok()?, h.parse::<f32>().ok()?]))).unwrap_or([1400.0, 860.0])).with_title("VintageCoder").with_app_id("vintagecoder"))), ..Default::default() };
    eframe::run_native("VintageCoder", opts, Box::new(move |cc| Ok(Box::new(app::App::new(cc, arg.clone())))))
}
