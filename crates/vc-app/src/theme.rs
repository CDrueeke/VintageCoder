// SPDX-License-Identifier: GPL-3.0-or-later
//! Darstellung: Farbschema (dunkel/hell/System), Oberflächen-Zoom und Schriftgröße des Editors.
//! Gespeichert in `~/.config/vintagecoder/view.json`. Die Farben der Oberfläche kommen von egui; die eigenen Farben
//! (Syntax, Meldungen, Zeichenflächen) liefern die Funktionen hier passend zum aktiven Schema.
use eframe::egui::{self, Color32};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static LIGHT: AtomicBool = AtomicBool::new(false);

/// Aktives Schema hell? (wird jedes Bild aus den egui-Einstellungen übernommen)
pub fn is_light() -> bool { LIGHT.load(Ordering::Relaxed) }
pub fn set_light(l: bool) { LIGHT.store(l, Ordering::Relaxed); }

pub fn ok() -> Color32 { crate::ui::tokens::t().gr }
pub fn warn() -> Color32 { crate::ui::tokens::t().am }
pub fn bad() -> Color32 { crate::ui::tokens::t().rd }
pub fn accent() -> Color32 { crate::ui::tokens::t().cy }
/// Gedämpfter Text (Zeilennummern, Spaltenköpfe)
pub fn dim() -> Color32 { crate::ui::tokens::t().mute }
/// Beschriftung neben Vorschaubildern: auf dunklem Grund hellgrau, auf hellem dunkelgrau
pub fn label_gray() -> Color32 { if is_light() { Color32::from_gray(90) } else { Color32::LIGHT_GRAY } }
/// Rahmen um die Auswahl auf Zeichenflächen und Paletten
pub fn select() -> Color32 { if is_light() { Color32::BLACK } else { Color32::WHITE } }
/// Hintergrund für Codeblöcke (Hilfe, Tracker): `level` 0 = Fläche, 1 = Inline-Code
pub fn code_bg(level: u8) -> Color32 {
    let k = crate::ui::tokens::t();
    if level == 0 { k.bg2 } else { k.bg3 }
}
/// Feine Linien und abgeblendete Zellen im Tracker (Grauwert dunkel → heller Gegenwert)
pub fn grid(dark_gray: u8) -> Color32 { if is_light() { Color32::from_gray(255 - dark_gray.saturating_sub(10).max(0)) } else { Color32::from_gray(dark_gray) } }

// ---- Einstellungen
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme { Dark, Light, System }

impl Scheme {
    fn key(self) -> &'static str { match self { Scheme::Dark => "dark", Scheme::Light => "light", Scheme::System => "system" } }
    fn parse(s: &str) -> Scheme { match s { "light" => Scheme::Light, "system" => Scheme::System, _ => Scheme::Dark } }
    fn preference(self) -> egui::ThemePreference { match self { Scheme::Dark => egui::ThemePreference::Dark, Scheme::Light => egui::ThemePreference::Light, Scheme::System => egui::ThemePreference::System } }
}

pub const ZOOM_RANGE: (f32, f32) = (0.7, 2.5);
/// Regler für den Oberflächen-Zoom. Beim Ziehen wird der Wert erst beim Loslassen übernommen: würde der Zoom live
/// wechseln, wanderte der Regler unter der Maus weg und das Fenster zappelte. Eingabe per Text/Tastatur gilt sofort.
pub fn zoom_slider(ui: &mut egui::Ui, zoom: &mut f32) -> egui::Response {
    let id = ui.id().with("zoom_pending");
    let mut v = ui.data(|d| d.get_temp::<f32>(id)).unwrap_or(*zoom);
    let r = ui.add(egui::Slider::new(&mut v, ZOOM_RANGE.0..=ZOOM_RANGE.1)
        .custom_formatter(|v, _| format!("{:.0} %", v * 100.0))
        .custom_parser(|t| t.trim().trim_end_matches('%').trim().parse::<f64>().ok().map(|v| v / 100.0)));
    if r.dragged() { ui.data_mut(|d| d.insert_temp(id, v)); }
    else {
        if r.changed() || r.drag_stopped() { *zoom = v.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1); }
        ui.data_mut(|d| d.remove_temp::<f32>(id));
    }
    r
}

pub const FONT_RANGE: (f32, f32) = (9.0, 36.0);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct View { pub scheme: Scheme, pub zoom: f32, pub font: f32 }

/// Schriftgröße des Editors, solange nichts gewählt ist
pub const DEFAULT_FONT: f32 = 13.0;

impl Default for View { fn default() -> Self { View { scheme: Scheme::Dark, zoom: 1.0, font: DEFAULT_FONT } } }

fn file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("view.json")) }

impl View {
    pub fn load() -> View {
        let mut v = View::default();
        let j = file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());
        if let Some(j) = j {
            if let Some(s) = j["scheme"].as_str() { v.scheme = Scheme::parse(s); }
            if let Some(z) = j["zoom"].as_f64() { v.zoom = (z as f32).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1); }
            if let Some(f) = j["font"].as_f64() { v.font = (f as f32).clamp(FONT_RANGE.0, FONT_RANGE.1); }
        }
        // Selbsttests (Screenshots): VC_THEME=dark|light, VC_ZOOM=1.5, VC_FONT=18
        if std::env::var("VC_SHOT").is_ok() {
            if let Ok(s) = std::env::var("VC_THEME") { v.scheme = Scheme::parse(&s); }
            if let Some(z) = std::env::var("VC_ZOOM").ok().and_then(|s| s.parse::<f32>().ok()) { v.zoom = z.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1); }
            if let Some(f) = std::env::var("VC_FONT").ok().and_then(|s| s.parse::<f32>().ok()) { v.font = f.clamp(FONT_RANGE.0, FONT_RANGE.1); }
        }
        v
    }

    /// Als Eintrag des Konfigurationsassistenten (`vc_ide::setup::ViewPrefs`, derselbe Inhalt wie `view.json`)
    pub fn to_prefs(&self) -> vc_ide::setup::ViewPrefs { vc_ide::setup::ViewPrefs { scheme: self.scheme.key().into(), zoom: self.zoom, font: self.font } }
    pub fn from_prefs(p: &vc_ide::setup::ViewPrefs) -> View { View { scheme: Scheme::parse(&p.scheme), zoom: p.zoom.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1), font: p.font.clamp(FONT_RANGE.0, FONT_RANGE.1) } }

    pub fn save(&self) {
        if std::env::var("VC_SHOT").is_ok() { return; }   // Selbsttests ändern keine Einstellungen
        if let Some(f) = file() {
            if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); }
            let _ = std::fs::write(f, format!("{}\n", serde_json::json!({ "scheme": self.scheme.key(), "zoom": self.zoom, "font": self.font })));
        }
    }

    /// Übernimmt Schema, Zoom und Schriftgröße in egui; billig genug für jedes Bild.
    pub fn apply(&self, ctx: &egui::Context) {
        if ctx.options(|o| o.theme_preference) != self.scheme.preference() { ctx.set_theme(self.scheme.preference()); }
        if (ctx.zoom_factor() - self.zoom).abs() > 0.001 { ctx.set_zoom_factor(self.zoom); }
        let size = self.font;
        let current = ctx.global_style().text_styles.get(&egui::TextStyle::Monospace).map_or(size, |f| f.size);
        if (current - size).abs() > 0.01 {
            ctx.all_styles_mut(|s| { if let Some(f) = s.text_styles.get_mut(&egui::TextStyle::Monospace) { f.size = size; } });
        }
        set_light(!ctx.global_style().visuals.dark_mode);
    }

    /// Strg/Cmd +, − und 0 ändern den Zoom von egui selbst; hier wird er übernommen und gespeichert.
    pub fn follow_zoom(&mut self, ctx: &egui::Context) -> bool {
        let z = ctx.zoom_factor();
        if (z - self.zoom).abs() > 0.001 { self.zoom = z.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1); self.save(); true } else { false }
    }
}
