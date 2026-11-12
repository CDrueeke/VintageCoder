// SPDX-License-Identifier: GPL-3.0-or-later
//! Export-Dialog der Studios (Mockup 12): Format als Segmentschalter, Adresse, Umfang als Kartenliste, Live-Vorschau mit Tabs und
//! drei Aktionen. Die Vorschau rechnet den aktuellen (auch ungespeicherten) Stand in einem Wegwerf-Ordner aus; geschrieben wird
//! nach `build/export/` erst durch „In Projekt speichern“. Dialekt und Komprimierung gibt es nicht.
use crate::bitmap::Env;
use crate::ui::tokens::t;
use crate::ui::widgets::{card, chip, segmented};
use eframe::egui::{self, RichText};
use std::path::Path;

/// Formate des Dialogs: (Kennung für `vc_media::project::export`, Beschriftung)
pub const FORMATS: &[(&str, &str)] = &[("asm", "Assembler"), ("inc", "Konstanten"), ("bin", "Binär"), ("prg", "PRG"), ("data", "BASIC DATA")];

/// Adresse in `$C000`, `0xC000` oder dezimaler Schreibweise
pub fn parse_addr(s: &str) -> Option<u16> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix('$').or_else(|| s.strip_prefix("0x")).or_else(|| s.strip_prefix("0X")) { return u16::from_str_radix(h, 16).ok(); }
    s.parse().ok()
}

/// Vorschlag für die Adresse je Art
pub fn default_addr(kind: &str) -> &'static str { match kind { "screen" => "$0400", "tilemap" | "tileset" => "$4000", "charset" => "$2000", "palette" => "$C000", _ => "$2000" } }

/// Hexauszug (16 Bytes je Zeile mit Adresse und ASCII-Spalte), höchstens `max` Bytes
pub fn hex_dump(bytes: &[u8], base: usize, max: usize) -> String {
    let mut out = String::new();
    for (i, ch) in bytes.chunks(16).take(max.div_ceil(16)).enumerate() {
        let hex: Vec<String> = ch.iter().map(|b| format!("{b:02x}")).collect();
        let asc: String = ch.iter().map(|b| if (0x20..0x7f).contains(b) { *b as char } else { '.' }).collect();
        out.push_str(&format!("{:04x}  {:<47}  {asc}\n", base + i * 16, hex.join(" ")));
    }
    if bytes.len() > max { out.push_str(&format!("… {} weitere Bytes\n", bytes.len() - max)); }
    out
}

/// Ergebnis einer Probe-Ausgabe
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Preview { pub files: Vec<(String, usize)>, pub text: String, pub hex: String, pub error: Option<String> }

/// Rechnet den Export von `media_text` (Inhalt der Mediendatei `rel`) in einem Wegwerf-Ordner aus.
/// Eigener Temp-Ordner je Aufruf (Prozess und Zähler), damit gleichzeitige Vorschauen
/// (mehrere Dialoge, parallele Tests) einander nicht löschen
fn unique_tmp(prefix: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)))
}

pub fn preview(data: &Path, rel: &str, media_text: &str, format: &str, label: &str, addr: Option<u16>) -> Preview {
    let tmp = unique_tmp("vc_export_prev");
    let _ = std::fs::remove_dir_all(&tmp);
    let run = || -> Result<Preview, String> {
        let file = tmp.join(rel);
        std::fs::create_dir_all(file.parent().ok_or("Pfad")?).map_err(|e| e.to_string())?;
        std::fs::write(&file, media_text).map_err(|e| e.to_string())?;
        let v = vc_media::project::export(&tmp, data, rel, format, Some(label).filter(|l| !l.is_empty()), addr, 10, 10)?;
        let files: Vec<(String, usize)> = v["files"].as_array().map(|a| a.iter().filter_map(|f| Some((f["path"].as_str()?.rsplit('/').next()?.to_string(), f["size"].as_u64()? as usize))).collect()).unwrap_or_default();
        let first = v["files"][0]["path"].as_str().map(|p| tmp.join(p));
        let bytes = first.and_then(|p| std::fs::read(p).ok()).unwrap_or_default();
        let is_text = matches!(format, "asm" | "inc" | "data");
        let text = if is_text { v["preview"].as_str().unwrap_or("").to_string() } else { format!("{}\n(Binärdaten: Reiter „Hex“)", v["preview"].as_str().unwrap_or("")) };
        Ok(Preview { files, text, hex: hex_dump(&bytes, addr.unwrap_or(0) as usize, 512), error: None })
    };
    let r = run().unwrap_or_else(|e| Preview { error: Some(e), ..Default::default() });
    let _ = std::fs::remove_dir_all(&tmp);
    r
}

/// BASIC-DATA-Zeilen (Format `data`) zur Vorschau
pub fn preview_data(data: &Path, rel: &str, media_text: &str, label: &str) -> Result<String, String> {
    let tmp = unique_tmp("vc_export_data");
    let _ = std::fs::remove_dir_all(&tmp);
    let r = (|| -> Result<String, String> {
        let file = tmp.join(rel);
        std::fs::create_dir_all(file.parent().ok_or("Pfad")?).map_err(|e| e.to_string())?;
        std::fs::write(&file, media_text).map_err(|e| e.to_string())?;
        let v = vc_media::project::export(&tmp, data, rel, "data", Some(label).filter(|l| !l.is_empty()), None, 10, 10)?;
        Ok(v["preview"].as_str().unwrap_or("").to_string())
    })();
    let _ = std::fs::remove_dir_all(&tmp);
    r
}

/// Schreibt wirklich nach `build/export/`; liefert die Ausgabedateien (Pfad, Größe)
pub fn write_export(env: &Env, rel: &str, format: &str, label: &str, addr: Option<u16>) -> Result<Vec<(String, usize)>, String> {
    let v = vc_media::project::export(env.root, env.data, rel, format, Some(label).filter(|l| !l.is_empty()), addr, 10, 10)?;
    Ok(v["files"].as_array().map(|a| a.iter().filter_map(|f| Some((f["path"].as_str()?.to_string(), f["size"].as_u64()? as usize))).collect()).unwrap_or_default())
}

/// Was der Nutzer im Dialog entschieden hat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Out { None, Close, Test, Save, Insert }

pub struct ExportDlg {
    pub rel: String,
    format: String,
    label: String,
    addr: String,
    tab: String,
    key: String,
    cache: Preview,
    data_tab: Option<Result<String, String>>,
}

impl ExportDlg {
    pub fn new(rel: &str, kind: &str, name: &str) -> ExportDlg {
        ExportDlg { rel: rel.into(), format: "asm".into(), label: name.replace('-', "_"), addr: default_addr(kind).into(), tab: "text".into(), key: String::new(), cache: Preview::default(), data_tab: None }
    }
    pub fn format(&self) -> &str { &self.format }
    pub fn label(&self) -> &str { &self.label }
    pub fn addr(&self) -> Option<u16> { parse_addr(&self.addr) }
    fn needs_addr(&self) -> bool { matches!(self.format.as_str(), "inc" | "prg") }

    /// Zeichnet den Dialog; `media_text` ist der aktuelle Stand der Mediendatei, `target` die Zielangabe fürs Chip
    pub fn show(&mut self, ctx: &egui::Context, env: &Env, media_text: &str, target: &str) -> Out {
        let k = t();
        let mut out = Out::None;
        let mut open = true;
        egui::Window::new(RichText::new("Export").strong()).id(egui::Id::new(("export-dlg", &self.rel))).open(&mut open).collapsible(false).resizable(true).default_width(640.0).default_height(560.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.rel).monospace().small().color(k.mute));
                chip(ui, target, k.cy);
            });
            ui.add_space(6.0);
            ui.label(RichText::new("AUSGABEFORMAT").small().color(k.dim));
            let mut f = self.format.clone();
            segmented(ui, &mut f, FORMATS);
            self.format = f;
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").small().color(k.dim));
                ui.add(egui::TextEdit::singleline(&mut self.label).desired_width(160.0).font(egui::TextStyle::Monospace));
                if self.needs_addr() {
                    ui.label(RichText::new(if self.format == "prg" { "Ladeadresse" } else { "Adresse" }).small().color(k.dim));
                    let bad = self.format == "prg" && self.addr().is_none() || !self.addr.trim().is_empty() && self.addr().is_none();
                    ui.add(egui::TextEdit::singleline(&mut self.addr).desired_width(80.0).font(egui::TextStyle::Monospace).text_color(if bad { k.rd } else { k.fg }));
                }
            });
            let addr = if self.needs_addr() { self.addr() } else { None };
            // Vorschau neu rechnen, wenn sich etwas geändert hat
            let key = format!("{}|{}|{:?}|{}", self.format, self.label, addr, media_text);
            if key != self.key {
                self.cache = if self.format == "prg" && addr.is_none() { Preview { error: Some("PRG braucht eine gültige Ladeadresse (z. B. $2000)".into()), ..Default::default() } } else { preview(env.data, &self.rel, media_text, &self.format, &self.label, addr) };
                self.data_tab = None;
                self.key = key;
            }
            ui.add_space(6.0);
            ui.label(RichText::new("UMFANG").small().color(k.dim));
            card(ui, false, |ui| {
                if self.cache.files.is_empty() { ui.label(RichText::new("–").color(k.dim)); }
                for (name, size) in &self.cache.files {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name).monospace().small());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new(crate::overview::fmt_size(*size)).monospace().small().color(k.mute)); });
                    });
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let mut tab = self.tab.clone();
                segmented(ui, &mut tab, &[("text", "Quelltext"), ("hex", "Hex"), ("data", "BASIC-DATA")]);
                self.tab = tab;
            });
            let body: String = match self.tab.as_str() {
                "hex" => if self.cache.hex.is_empty() { "(keine Binärdaten)".into() } else { self.cache.hex.clone() },
                "data" => {
                    if self.data_tab.is_none() { self.data_tab = Some(preview_data(env.data, &self.rel, media_text, &self.label)); }
                    match self.data_tab.as_ref() { Some(Ok(s)) => s.clone(), Some(Err(e)) => e.clone(), None => String::new() }
                }
                _ => self.cache.error.clone().unwrap_or_else(|| self.cache.text.clone()),
            };
            egui::Frame::new().fill(k.bg0).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
                egui::ScrollArea::both().max_height(220.0).auto_shrink([false, true]).show(ui, |ui| { ui.add(egui::Label::new(RichText::new(body).monospace().small()).extend()); });
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                match &self.cache.error {
                    Some(e) => { ui.label(RichText::new(format!("● {e}")).small().color(k.rd)); }
                    None => { ui.label(RichText::new(format!("● bereit · {}", crate::overview::fmt_size(self.cache.files.iter().map(|f| f.1).sum()))).small().color(k.gr)); }
                }
            });
            ui.add_space(6.0);
            let ok = self.cache.error.is_none();
            ui.horizontal(|ui| {
                if ui.add_enabled(ok, egui::Button::new("Im Emulator testen")).on_hover_text("Speichert, exportiert als Assembler, baut das Projekt und startet es").clicked() { out = Out::Test; }
                if ui.add_enabled(ok, egui::Button::new("In Projekt speichern")).on_hover_text("Schreibt die Dateien nach build/export/").clicked() { out = Out::Save; }
                if ui.add_enabled(ok, egui::Button::new(RichText::new("In aktiven Code einfügen").strong().color(k.on_cy)).fill(k.cy)).on_hover_text("Exportiert und fügt .include in den zuletzt benutzten Quelltext ein").clicked() { out = Out::Insert; }
            });
        });
        if !open { return Out::Close; }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { return Out::Close; }
        out
    }
}

/// Zeigt den Dialog (falls offen) und führt die Entscheidung aus. `save` speichert die Mediendatei vorher.
/// Liefert einen Statustext, wenn etwas geschehen ist.
pub fn drive(exp: &mut Option<ExportDlg>, ui: &mut egui::Ui, env: &Env, rel: &str, media_text: &str, target: &str, save: impl FnOnce() -> Result<(), String>) -> Option<String> {
    let mut dlg = exp.take()?;
    let out = dlg.show(ui.ctx(), env, media_text, target);
    match out {
        Out::None => { *exp = Some(dlg); None }
        Out::Close => None,
        o => {
            if let Err(e) = save() { *exp = Some(dlg); return Some(e); }
            let fmt = if o == Out::Test || o == Out::Insert { "asm".to_string() } else { dlg.format().to_string() };
            match write_export(env, rel, &fmt, dlg.label(), dlg.addr()) {
                Ok(files) => {
                    env.reqs.info(format!("Exportiert: {}", files.iter().map(|f| f.0.as_str()).collect::<Vec<_>>().join(", ")));
                    if o == Out::Insert { env.reqs.insert_asset(rel); }
                    if o == Out::Test { env.reqs.run_project(); }
                    if o == Out::Save { *exp = Some(dlg); }
                    Some(format!("exportiert ({fmt})"))
                }
                Err(e) => { env.reqs.error(format!("Export {rel}: {e}")); *exp = Some(dlg); Some(e) }
            }
        }
    }
}

/// Öffnet den Dialog, falls er geschlossen ist
/// Menü „Datei ▸ Exportieren …“: das sichtbare Studio öffnet seinen Export-Dialog (die Anwendung setzt es und löscht es nach dem Bild)
pub static WANT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Hat das Menü den Export angefordert? (setzt die Anforderung zurück)
pub fn wanted() -> bool { WANT.swap(false, std::sync::atomic::Ordering::Relaxed) }

pub fn open(exp: &mut Option<ExportDlg>, rel: &str, kind: &str, name: &str) { if exp.is_none() { *exp = Some(ExportDlg::new(rel, kind, name)); } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adressen_lesen() {
        assert_eq!(parse_addr("$C000"), Some(0xC000));
        assert_eq!(parse_addr(" 0x0400 "), Some(0x400));
        assert_eq!(parse_addr("8192"), Some(8192));
        assert_eq!(parse_addr("$zz"), None);
        assert_eq!(parse_addr("70000"), None);
        assert_eq!(parse_addr(""), None);
    }

    #[test]
    fn hexauszug() {
        let d: Vec<u8> = (0..20u8).map(|i| 0x40 + i).collect();
        let h = hex_dump(&d, 0x2000, 16);
        assert!(h.starts_with("2000  40 41 42"), "{h}");
        assert!(h.contains("@ABCDEFGHIJKLMNO"));
        assert!(h.contains("… 4 weitere Bytes"));
    }

    #[test]
    fn vorschau_gleichzeitig() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let med = vc_media::project::format_media(&serde_json::json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": "font", "frames": (0..256).map(|_| serde_json::json!({ "rows": vec!["00000000"; 8] })).collect::<Vec<_>>() }));
        let hs: Vec<_> = (0..8).map(|_| { let (d, m) = (data.clone(), med.clone()); std::thread::spawn(move || (0..20).map(|_| preview(&d, "gfx/font.vcchar", &m, "prg", "font", Some(0x2000))).all(|p| p.error.is_none() && p.files.len() == 1)) }).collect();
        for h in hs { assert!(h.join().unwrap(), "gleichzeitige Vorschau schlug fehl"); }
    }

    #[test]
    fn formate() { assert_eq!(FORMATS.len(), 5); assert!(FORMATS.iter().all(|f| !f.1.is_empty())); }

    #[test]
    fn vorschau_ohne_projektordner() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let med = vc_media::project::format_media(&serde_json::json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": "font", "frames": (0..256).map(|_| serde_json::json!({ "rows": vec!["00000000"; 8] })).collect::<Vec<_>>() }));
        let p = preview(&data, "gfx/font.vcchar", &med, "asm", "font", None);
        assert!(p.error.is_none(), "{:?}", p.error);
        assert!(p.files.iter().any(|f| f.0.ends_with(".asm")));
        assert!(p.text.contains("font"), "{}", p.text);
        let p = preview(&data, "gfx/font.vcchar", &med, "prg", "font", Some(0x2000));
        assert!(p.error.is_none(), "{:?}", p.error);
        assert_eq!(p.files[0].1, 2048 + 2);
        assert!(p.hex.starts_with("2000"));
        let p = preview(&data, "gfx/font.vcchar", &med, "prg", "font", None);
        assert!(p.error.is_some());
        assert!(preview_data(&data, "gfx/font.vcchar", &med, "font").is_ok());
    }
}
