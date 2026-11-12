// SPDX-License-Identifier: GPL-3.0-or-later
//! Gemeinsame Anatomie der Studios (Sprite/PMG, Zeichensatz, Bildschirm, Kacheln): Kopfzeile mit Dateiname und Chips,
//! Werkzeugleiste, links Liste oder Matrix, Mitte Canvas, rechts Eigenschaften, Fuß mit Export-Streifen und Status.
//! Jede Funktion zeigt ein Feld im übergebenen `ui`; die Reihenfolge der Aufrufe ist: `head`, `tools`, `foot`, `left`, `right`, `center`.
use super::tokens::{t, tint};
use super::widgets::chip;
use eframe::egui::{self, Color32, RichText};

/// Inhalt der Kopfzeile: Dateiname, ungespeichert-Punkt und Chips (Text, Farbe)
pub struct Head<'a> { pub file: &'a str, pub dirty: bool, pub chips: &'a [(String, Color32)] }

/// Inhalt des Fußes: Export-Streifen und Status
pub struct Foot<'a> {
    /// Formate, die der Export anbietet (Kürzel wie „ASM“, „BIN“)
    pub formats: &'a [&'a str],
    /// Letzte Ausgabedatei (relativ zum Projekt) und ob sie zum gespeicherten Stand passt
    pub out: Option<(&'a str, bool)>,
    /// Rechts: Ziel und Zusatzangaben (z. B. „Ziel: C64 PAL“)
    pub info: &'a str,
    pub status: &'a str,
}

/// Was der Nutzer im Fuß angestoßen hat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootAct { None, OpenExport }

fn bar(ui: &mut egui::Ui, panel: egui::Panel, fill: Color32, margin: egui::Margin, body: impl FnOnce(&mut egui::Ui)) {
    panel.frame(egui::Frame::new().fill(fill).inner_margin(margin)).show_inside(ui, body);
}

/// Kopfzeile: Dateiname (Festbreite), Punkt bei ungespeicherten Änderungen, Chips für System und Profil
pub fn head(ui: &mut egui::Ui, id: &str, h: &Head) {
    let k = t();
    bar(ui, egui::Panel::top(egui::Id::new(("st-head", id))), k.bg1, egui::Margin::symmetric(10, 6), |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(h.file).font(egui::FontId::new(14.0, crate::fonts::mono_strong())).color(k.fg));
            if h.dirty { ui.label(RichText::new("●").small().color(k.am)).on_hover_text("Ungespeicherte Änderungen"); }
            for (text, c) in h.chips { chip(ui, text, *c); }
        });
    });
}

/// Werkzeugleiste unter der Kopfzeile (umbrechend)
pub fn tools(ui: &mut egui::Ui, id: &str, body: impl FnOnce(&mut egui::Ui)) {
    let k = t();
    bar(ui, egui::Panel::top(egui::Id::new(("st-tools", id))), k.bg2, egui::Margin::symmetric(10, 4), |ui| { ui.spacing_mut().slider_width = 90.0; ui.horizontal_wrapped(body); });
}

/// Fuß: Export-Streifen (Formate, Ausgabedatei, Knopf „Export …“) und Status
pub fn foot(ui: &mut egui::Ui, id: &str, f: &Foot) -> FootAct {
    let k = t();
    let mut act = if crate::exportdlg::wanted() { FootAct::OpenExport } else { FootAct::None };
    bar(ui, egui::Panel::bottom(egui::Id::new(("st-foot", id))), k.bg1, egui::Margin::symmetric(10, 5), |ui| {
        // Schmal: umbrechen, Status und Ziel folgen als eigene Zeile; breit: Status rechtsbündig in einer Zeile
        let narrow = ui.available_width() < 640.0;
        let mut strip = |ui: &mut egui::Ui| {
            ui.label(RichText::new("EXPORT").small().color(k.dim));
            for fm in f.formats { chip(ui, fm, k.cy); }
            if let Some((path, fresh)) = f.out {
                ui.add(egui::Label::new(RichText::new(path).monospace().small().color(k.mute)).truncate());
                chip(ui, if fresh { "aktuell" } else { "veraltet" }, if fresh { k.gr } else { k.am });
            } else {
                ui.label(RichText::new("noch nicht exportiert").small().color(k.dim));
            }
            if ui.small_button("Export …").on_hover_text("Format, Adresse und Vorschau wählen").clicked() { act = FootAct::OpenExport; }
        };
        if narrow {
            ui.horizontal_wrapped(|ui| strip(ui));
            if !f.status.is_empty() || !f.info.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    if !f.info.is_empty() { ui.label(RichText::new(f.info).small().color(k.dim)); }
                    if !f.status.is_empty() { ui.label(RichText::new(f.status).small().color(k.mute)); }
                });
            }
        } else {
            ui.horizontal(|ui| {
                strip(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !f.status.is_empty() { ui.label(RichText::new(f.status).small().color(k.mute)); }
                    if !f.info.is_empty() { ui.label(RichText::new(f.info).small().color(k.dim)); }
                });
            });
        }
    });
    act
}

/// Gesamtbreite, ab der die linke Spalte (Liste/Matrix) neben der Mitte steht; darunter klappt sie in eine Zeile über der Mitte ein
pub const LEFT_MIN_TOTAL: f32 = 560.0;
/// Verbleibende Breite (nach der linken Spalte), ab der die rechte Spalte (Eigenschaften) seitlich steht; darunter klappt sie unter die Mitte
pub const RIGHT_MIN_TOTAL: f32 = 560.0;

/// Wo eine Seitenspalte bei der gegebenen Breite sitzt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidePlace { Side, Folded }

/// Linke Spalte: `total` = freie Breite des Studios
pub fn left_place(total: f32) -> SidePlace { if total >= LEFT_MIN_TOTAL { SidePlace::Side } else { SidePlace::Folded } }
/// Rechte Spalte: `avail` = freie Breite nach der linken Spalte, `min` = ihre Mindestbreite (0 = Standard)
pub fn right_place(avail: f32, min: f32) -> SidePlace { if avail >= RIGHT_MIN_TOTAL.max(min + 200.0) { SidePlace::Side } else { SidePlace::Folded } }

/// Ganzzahliger Zoom, bei dem `cells` (Breite, Höhe in Zoomeinheiten) in `avail` passen; `reserve` = Rand und Bildlaufleisten.
/// Das Ergebnis liegt in `min..=max` (ist der Platz kleiner als `min`, gewinnt `min` und die Fläche scrollt).
pub fn fit_zoom(avail: egui::Vec2, cells: egui::Vec2, reserve: egui::Vec2, min: f32, max: f32) -> f32 {
    let f = |a: f32, c: f32| if c > 0.0 { a / c } else { f32::INFINITY };
    let z = f(avail.x - reserve.x, cells.x).min(f(avail.y - reserve.y, cells.y));
    if !z.is_finite() { return min.max(1.0).min(max); }
    z.floor().clamp(min, max.max(min))
}

/// Eingeklappte Spalte: Überschrift zum Aufklappen, Inhalt begrenzt scrollbar
fn folded(ui: &mut egui::Ui, id: egui::Id, title: &str, bottom: bool, max_h: f32, body: impl FnOnce(&mut egui::Ui)) {
    let k = t();
    let panel = if bottom { egui::Panel::bottom(id) } else { egui::Panel::top(id) };
    bar(ui, panel, k.bg2, egui::Margin::symmetric(8, 3), |ui| {
        egui::CollapsingHeader::new(RichText::new(title).small().color(k.dim)).id_salt(id.with("fold")).default_open(false).show(ui, |ui| {
            egui::ScrollArea::vertical().max_height(max_h).auto_shrink([false, true]).show(ui, body);
        });
    });
}

/// Größte Breite einer Seitenspalte: ein Drittel des Platzes, damit die Mitte nie verschwindet
pub fn side_max(ui: &egui::Ui) -> f32 { (ui.available_width() * 0.33).max(60.0) }

/// Linke Spalte (Liste oder Matrix)
pub fn left(ui: &mut egui::Ui, id: &str, width: f32, body: impl FnOnce(&mut egui::Ui)) {
    let k = t();
    if left_place(ui.available_width()) == SidePlace::Folded { let mh = (ui.available_height() * 0.4).max(80.0); folded(ui, egui::Id::new(("st-left-f", id)), "Liste", false, mh, body); return; }
    let max = side_max(ui);
    bar(ui, egui::Panel::left(egui::Id::new(("st-left", id))).default_size(width.min(max)).size_range(60.0..=max), k.bg2, egui::Margin::same(8), body);
}

/// Rechte Spalte (Eigenschaften), scrollbar
pub fn right(ui: &mut egui::Ui, id: &str, width: f32, body: impl FnOnce(&mut egui::Ui)) {
    let k = t();
    if right_place(ui.available_width(), 0.0) == SidePlace::Folded { let mh = (ui.available_height() * 0.45).max(100.0); folded(ui, egui::Id::new(("st-right-f", id)), "Eigenschaften", true, mh, body); return; }
    let max = side_max(ui);
    bar(ui, egui::Panel::right(egui::Id::new(("st-right", id))).default_size(width.min(max)).size_range(60.0..=max), k.bg2, egui::Margin::same(8), |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, body);
    });
}

/// Rechte Spalte mit fester Mindestbreite: ist das Fenster zu schmal, scrollt der Inhalt (auch waagerecht), statt abgeschnitten zu werden
pub fn right_min(ui: &mut egui::Ui, id: &str, width: f32, min: f32, body: impl FnOnce(&mut egui::Ui)) {
    let k = t();
    if right_place(ui.available_width(), min) == SidePlace::Folded { let mh = (ui.available_height() * 0.45).max(100.0); folded(ui, egui::Id::new(("st-right-f", id)), "Eigenschaften", true, mh, body); return; }
    let max = side_max(ui).max(min);
    bar(ui, egui::Panel::right(egui::Id::new(("st-right", id))).default_size(width.clamp(min, max)).size_range(min..=max), k.bg2, egui::Margin::same(8), |ui| {
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| { ui.set_min_width(min - 16.0); body(ui) });
    });
}

/// Mitte: Canvas auf der tiefsten Fläche, scrollbar in beide Richtungen
pub fn center(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    egui::CentralPanel::default().frame(egui::Frame::new().fill(t().bg0).inner_margin(egui::Margin::same(8))).show_inside(ui, |ui| {
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, body);
    });
}

/// Überschrift eines Blocks in den Seitenspalten
pub fn caption(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text.to_uppercase()).small().color(t().dim));
}

/// Zeile der Pixelwerttabelle: Farbfeld (oder durchgestrichen), Wert als Bitfolge, Name und Registeradresse; liefert, ob angeklickt
pub fn value_row(ui: &mut egui::Ui, selected: bool, color: Option<Color32>, bits: &str, name: &str, reg: Option<&str>) -> bool {
    let k = t();
    let mut clicked = false;
    let fill = if selected { tint(k.cy) } else { Color32::TRANSPARENT };
    let resp = egui::Frame::new().fill(fill).corner_radius(4.0).inner_margin(egui::Margin::symmetric(4, 2)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            match color {
                Some(c) => { ui.painter().rect_filled(r, 3.0, c); }
                None => { ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, k.dim), egui::StrokeKind::Inside); ui.painter().line_segment([r.left_bottom(), r.right_top()], egui::Stroke::new(1.0, k.dim)); }
            }
            ui.label(RichText::new(bits).monospace().small().color(k.cy));
            ui.label(RichText::new(name).color(if selected { k.fg } else { k.mute }));
            if let Some(reg) = reg { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new(reg).monospace().small().color(k.am)); }); }
        });
    }).response;
    if ui.interact(resp.rect, ui.id().with(("value", bits, name)), egui::Sense::click()).clicked() { clicked = true; }
    clicked
}

/// Bits eines Pixelwerts als Text, `bpp` Stellen: `value_bits(2, 2)` → „10“
pub fn value_bits(value: usize, bpp: usize) -> String { (0..bpp.max(1)).rev().map(|b| if value >> b & 1 == 1 { '1' } else { '0' }).collect() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seitenspalten_lassen_der_mitte_platz() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        for (w, h) in [(330.0, 500.0), (640.0, 480.0), (1100.0, 760.0)] {
            let mut mid = 0.0;
            for _ in 0..3 {
                let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(w, h))), ..Default::default() }, |ui| {
                    foot(ui, "t", &Foot { formats: &["ASM", "INC", "BIN"], out: Some(("build/export/level.asm", true)), info: "Ziel: C64 PAL", status: "gespeichert" });
                    left(ui, "t", 400.0, |ui| { ui.label("Liste"); });
                    right(ui, "t", 400.0, |ui| { ui.label("Eigenschaften"); });
                    center(ui, |ui| { mid = ui.max_rect().width(); });
                });
                o.textures_delta.clear();
            }
            assert!(mid >= w * 0.30, "Mitte hat bei {w}x{h} nur {mid} Pixel");
        }
    }

    #[test]
    fn zoom_passt_in_den_platz() {
        let v = egui::vec2;
        // 24×21 Zellen à 2 Einheiten Breite: 800×600 frei, 20 Rand → 780/48=16, 580/21=27 → 16
        assert_eq!(fit_zoom(v(800.0, 600.0), v(48.0, 21.0), v(20.0, 20.0), 6.0, 48.0), 16.0);
        // winziger Platz: Mindestzoom gewinnt
        assert_eq!(fit_zoom(v(100.0, 100.0), v(48.0, 21.0), v(20.0, 20.0), 6.0, 48.0), 6.0);
        // riesiger Platz: Höchstzoom
        assert_eq!(fit_zoom(v(4000.0, 3000.0), v(8.0, 8.0), v(0.0, 0.0), 6.0, 48.0), 48.0);
        // nie größer als der Platz (ganzzahlig abgerundet)
        let z = fit_zoom(v(500.0, 400.0), v(40.0, 25.0), v(0.0, 0.0), 4.0, 48.0);
        assert_eq!(z, 12.0); assert!(z * 40.0 <= 500.0 && z * 25.0 <= 400.0);
        // leerer Inhalt, unendlicher Platz: kein Absturz
        assert_eq!(fit_zoom(v(f32::INFINITY, f32::INFINITY), v(0.0, 0.0), v(0.0, 0.0), 6.0, 48.0), 6.0);
    }

    #[test]
    fn spalten_klappen_ein() {
        assert_eq!(left_place(1100.0), SidePlace::Side);
        assert_eq!(left_place(480.0), SidePlace::Folded, "halber Editorbereich: Liste eingeklappt");
        assert_eq!(left_place(LEFT_MIN_TOTAL), SidePlace::Side);
        assert_eq!(right_place(700.0, 0.0), SidePlace::Side);
        assert_eq!(right_place(480.0, 0.0), SidePlace::Folded, "halber Editorbereich: Eigenschaften eingeklappt");
        assert_eq!(right_place(590.0, 400.0), SidePlace::Folded, "Mindestbreite 400 braucht mehr Platz");
        assert_eq!(right_place(700.0, 300.0), SidePlace::Side, "Kacheln: 300 Mindestbreite passt bei 700");
    }

    #[test]
    fn eingeklappt_bleibt_benutzbar() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        for (w, h) in [(480.0, 600.0), (330.0, 500.0)] {
            let mut mid = (0.0, 0.0);
            for _ in 0..3 {
                let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(w, h))), ..Default::default() }, |ui| {
                    foot(ui, "f", &Foot { formats: &["ASM"], out: None, info: "", status: "" });
                    left(ui, "f", 400.0, |ui| { ui.label("Liste"); });
                    right(ui, "f", 400.0, |ui| { ui.label("Eigenschaften"); });
                    center(ui, |ui| { mid = (ui.max_rect().width(), ui.max_rect().height()); });
                });
                o.textures_delta.clear();
            }
            assert!(mid.0 >= w - 20.0 && mid.1 >= 150.0, "Mitte bei {w}x{h}: {mid:?}");
        }
    }

    #[test]
    fn bits_als_text() {
        assert_eq!(value_bits(2, 2), "10");
        assert_eq!(value_bits(1, 1), "1");
        assert_eq!(value_bits(0, 2), "00");
        assert_eq!(value_bits(5, 8), "00000101");
    }
}
