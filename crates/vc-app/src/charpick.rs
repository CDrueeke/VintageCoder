// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialog „Zeichen einfügen“: PETSCII/Bildschirmcode (Commodore) bzw. ATASCII/ANTIC-Code (Atari) aus den Original-Zeichensätzen
//! (aus den ROMs des Anwenders) wählen und als `{name}`-Sonderzeichen, Wert, `.byte`-Zeile usw. in den Quelltext einfügen.
use eframe::egui::{self, pos2, vec2, Align2, Color32, ColorImage, Rect, Sense, Stroke, TextureHandle, TextureOptions};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use vc_asm::tokens::{self, Names};

const CELL: f32 = 28.0;
const SCALE: f32 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode { InString, Value, ByteDir, Decimal, ScreenCode }

pub enum Out { None, Insert(String), Close }

struct SetEntry { label: String, file: String }

pub struct CharPick {
    atari: bool,
    sys: String,
    sets: Vec<SetEntry>,
    set: usize,
    /// 0 = PETSCII / ATASCII, 1 = Bildschirmcode / ANTIC-Code
    tab: usize,
    cache: HashMap<String, Rc<Vec<TextureHandle>>>,
    sel: u8,
    basket: Vec<u8>,
    recent: Vec<u8>,
    search: String,
    mode: Mode,
    mode_user: bool,
    keep_open: bool,
    inverse: bool,
    data: std::path::PathBuf,
    #[allow(dead_code)]
    pub error: Option<String>,
}

fn abbr(atari: bool, code: u8) -> Option<String> {
    let names = tokens::names_of(if atari { Names::Atascii } else { Names::Petscii });
    let (n, _, _) = names.iter().find(|(_, c, _)| *c == code)?;
    if matches!(*n, "space" | "shift space") { return None; }
    Some(match *n {
        "rvs on" => "RVS".into(), "rvs off" => "OFF".into(), "shift ret" => "S-RET".into(), "orange" => "ORG".into(), "lower" => "a/A".into(), "upper" => "A/a".into(),
        "del line" => "DEL".into(), "ins line" => "INS".into(), "clr tab" => "CT".into(), "set tab" => "ST".into(), "del char" => "DCH".into(), "ins char" => "ICH".into(),
        "up" => "UP".into(), "down" => "DN".into(), "left" => "LT".into(), "right" => "RT".into(),
        other => other.to_uppercase(),
    })
}

/// Glyph-Nummer im Zeichensatz-Bild (und ob invers) für einen Code der Tabelle; `None` = Steuerzeichen/unbelegt
fn glyph_of(atari: bool, tab: usize, code: u8) -> Option<(usize, bool)> {
    if atari {
        if tab == 1 { return Some(((code & 0x7F) as usize, code >= 0x80)); }
        if abbr(true, code).is_some() { return None; }
        let b = code & 0x7F;
        let i = if b < 0x20 { b + 0x40 } else if b < 0x60 { b - 0x20 } else { b };
        return Some((i as usize, code >= 0x80));
    }
    if tab == 1 { return Some((code as usize, false)); }
    let i = match code {
        0x00..=0x1F | 0x80..=0x9F => return None,
        0x20..=0x3F => code,
        0x40..=0x5F => code - 0x40,
        0x60..=0x7F => code - 0x20,
        0xA0..=0xBF => code - 0x40,
        0xC0..=0xFE => code - 0x80,
        0xFF => 0x5E,
    };
    Some((i as usize, false))
}

/// Bildschirmcode-Wert zu einem Code der ersten Tabelle (für „Einfügen als Bildschirmcode“)
fn screen_value(atari: bool, tab: usize, code: u8) -> Option<u8> {
    glyph_of(atari, tab, code).map(|(i, inv)| if atari { i as u8 | if inv { 0x80 } else { 0 } } else { i as u8 })
}

/// Zeichen, das man im Quelltext einer Zeichenkette direkt schreiben kann (wird vom Assembler wieder in diesen Code umgewandelt)
fn literal_of(atari: bool, tab: usize, code: u8) -> Option<char> {
    if tab == 1 { return None; }
    let c = if atari {
        match code { 0x20..=0x7A => code as char, _ => return None }
    } else {
        match code { 0x20..=0x3F => code as char, 0x41..=0x5A => (code + 0x20) as char, 0xC1..=0xDA => (code - 0x80) as char, _ => return None }
    };
    if c == '"' || c == '{' || c == '}' { None } else { Some(c) }
}

fn token_of(atari: bool, tab: usize, code: u8) -> String {
    if tab == 0 {
        let names = tokens::names_of(if atari { Names::Atascii } else { Names::Petscii });
        if let Some((n, _, _)) = names.iter().find(|(_, c, _)| *c == code) { return format!("{{{n}}}"); }
    }
    format!("{{${code:02x}}}")
}

/// Text für einen Satz Codes, je nach Einfügeart
pub fn compose(atari: bool, tab: usize, mode: Mode, codes: &[u8]) -> String {
    match mode {
        Mode::InString => {
            let mut out = String::new();
            let mut i = 0;
            while i < codes.len() {
                let c = codes[i];
                let mut n = 1; while i + n < codes.len() && codes[i + n] == c { n += 1; }
                match literal_of(atari, tab, c) {
                    Some(ch) => out.extend(std::iter::repeat(ch).take(n)),
                    None => { let t = token_of(atari, tab, c); if n > 1 { out.push_str(&format!("{}*{n}}}", &t[..t.len() - 1])); } else { out.push_str(&t); } }
                }
                i += n;
            }
            out
        }
        Mode::Value => codes.iter().map(|c| format!("${c:02x}")).collect::<Vec<_>>().join(", "),
        Mode::ByteDir => format!(".byte {}", codes.iter().map(|c| format!("${c:02x}")).collect::<Vec<_>>().join(", ")),
        Mode::Decimal => codes.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", "),
        Mode::ScreenCode => codes.iter().filter_map(|c| screen_value(atari, tab, *c)).map(|v| format!("${v:02x}")).collect::<Vec<_>>().join(", "),
    }
}

fn describe(atari: bool, tab: usize, code: u8) -> (String, String) {
    // (Name, Tastatur)
    if tab == 0 {
        let names = tokens::names_of(if atari { Names::Atascii } else { Names::Petscii });
        if let Some((n, _, d)) = names.iter().find(|(_, c, _)| *c == code) { return (format!("{d}  ·  {{{n}}}"), String::new()); }
    }
    let name = match literal_of(atari, tab, code) { Some(c) => format!("im Quelltext tippbar als »{c}«"), None => if glyph_of(atari, tab, code).is_some() { "Grafikzeichen".into() } else { "unbelegt".into() } };
    let key = if tab == 1 { String::new() } else if atari {
        match code { 0x01..=0x1A => format!("Strg+{}", (b'A' + code - 1) as char), 0x00 => "Strg+,".into(), _ => String::new() }
    } else {
        match code { 0xC1..=0xDA => format!("Umschalt+{}", (code - 0x80) as char), 0x41..=0x5A => format!("{}", (code) as char), _ => String::new() }
    };
    (name, key)
}

impl CharPick {
    pub fn new(sys: &str, data: &Path) -> CharPick {
        let atari = sys == "atari8";
        let want = if sys == "mega65" { "c64" } else { sys };
        // Zeichensätze aus den ROMs des Anwenders (Verweise `rom:system/rolle#satz`); fehlt jedes ROM, bleibt die Liste leer
        let mut sets: Vec<SetEntry> = vec![];
        for c in vc_media::project::std_charset_list(data) {
            let hires8 = c.profile.ends_with("char.hires") || c.profile.ends_with("char.antic2");
            if c.system == want && hires8 && c.status != vc_media::Status::Missing && !sets.iter().any(|x| x.file == c.reference) {
                sets.push(SetEntry { label: c.label, file: c.reference });
            }
        }
        let sets_empty = sets.is_empty();
        let set = sets.iter().position(|s| s.file.ends_with("#upper") || s.file.ends_with("#standard")).unwrap_or(0);
        CharPick { atari, sys: sys.to_string(), sets, set, tab: 0, cache: HashMap::new(), sel: if atari { 0x14 } else { 0xC1 }, basket: vec![], recent: vec![], search: String::new(),
                   mode: Mode::Value, mode_user: false, keep_open: true, inverse: true, data: data.to_path_buf(), error: if sets_empty { Some("Kein Zeichensatz-ROM gefunden – ROMs einrichten (Einstellungen → ROMs)".into()) } else { None } }
    }

    pub fn system(&self) -> &str { &self.sys }

    fn load(&mut self, ctx: &egui::Context, file: &str) -> Option<Rc<Vec<TextureHandle>>> {
        if let Some(c) = self.cache.get(file) { return Some(c.clone()); }
        let profile = vc_media::project::std_charset_list(&self.data).into_iter().find(|c| c.reference == file && c.system == if self.sys == "mega65" { "c64" } else { self.sys.as_str() })?.profile;
        let v: Value = vc_media::project::std_charset_media(&profile, file).ok()?;
        let frames = v["frames"].as_array()?;
        let tex: Vec<TextureHandle> = frames.iter().enumerate().map(|(i, f)| {
            let mut px = vec![0u8; 8 * 8 * 4];
            for (y, r) in f["rows"].as_array().into_iter().flatten().take(8).enumerate() {
                for (x, c) in r.as_str().unwrap_or("").chars().take(8).enumerate() { if c != '0' { let o = (y * 8 + x) * 4; px[o..o + 4].copy_from_slice(&[255, 255, 255, 255]); } }
            }
            ctx.load_texture(format!("charpick-{file}-{i}"), ColorImage::from_rgba_unmultiplied([8, 8], &px), TextureOptions::NEAREST)
        }).collect();
        let rc = Rc::new(tex);
        self.cache.insert(file.to_string(), rc.clone());
        Some(rc)
    }

    fn paint_glyph(ui: &egui::Ui, tex: &TextureHandle, r: Rect, ink: Color32) {
        ui.painter().image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), ink);
    }

    /// `in_string`: Der Cursor steht in einer Zeichenkette einer Textdirektive (dann ist „In Zeichenkette“ die Vorgabe).
    pub fn show(&mut self, ctx: &egui::Context, in_string: bool) -> Out {
        let mut out = Out::None;
        let atari = self.atari;
        if !self.mode_user { self.mode = if in_string { Mode::InString } else { Mode::Value }; }
        let tabs: [&str; 2] = if atari { ["ATASCII", "ANTIC-Code"] } else { ["PETSCII", "Bildschirmcode"] };
        let file = self.sets.get(self.set).map(|s| s.file.clone());
        let tex = file.as_deref().and_then(|f| self.load(ctx, f));
        let mut open = true;
        let (accent, dim) = (crate::theme::accent(), crate::theme::dim());
        egui::Window::new("Zeichen einfügen").title_bar(false).collapsible(false).resizable(false).frame(crate::ui::dialog::frame()).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            let k = crate::ui::tokens::t();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Zeichen einfügen").font(egui::FontId::new(16.0, crate::fonts::ui_strong())).color(k.fg));
                crate::ui::widgets::chip(ui, if atari { "ATASCII" } else { "PETSCII" }, k.cy);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.button("Schließen").clicked() { open = false; } });
            });
            ui.label(egui::RichText::new("Zeichen des Original-Zeichensatzes des Zielsystems · Doppelklick fügt sofort ein").small().color(dim));
            ui.separator();
            ui.horizontal(|ui| {
                for (i, t) in tabs.iter().enumerate() { if ui.selectable_label(self.tab == i, *t).clicked() { self.tab = i; } }
                ui.label(egui::RichText::new("Reiter und Vorgabe folgen dem Ziel des Projekts").small().color(dim));
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Zeichensatz").small().color(dim));
                let cur = self.sets.get(self.set).map(|s| s.label.clone()).unwrap_or_else(|| "(keiner gefunden)".into());
                egui::ComboBox::from_id_salt("charpick-set").width(330.0).selected_text(cur).show_ui(ui, |ui| {
                    for i in 0..self.sets.len() { let l = self.sets[i].label.clone(); ui.selectable_value(&mut self.set, i, l); }
                });
                if atari {
                    ui.selectable_value(&mut self.inverse, false, "Normal");
                    ui.selectable_value(&mut self.inverse, true, "Invers anzeigen");
                } else if let Some(cur) = self.sets.get(self.set) {
                    let (upper, lower) = (cur.file.replace("#lower", "#upper"), cur.file.replace("#upper", "#lower"));
                    let is_upper = cur.file.ends_with("#upper");
                    if cur.file.ends_with("#upper") || cur.file.ends_with("#lower") {
                        if ui.selectable_label(is_upper, "Groß/Grafik").clicked() { if let Some(i) = self.sets.iter().position(|s| s.file == upper) { self.set = i; } }
                        if ui.selectable_label(!is_upper, "Klein/Groß").clicked() { if let Some(i) = self.sets.iter().position(|s| s.file == lower) { self.set = i; } }
                    }
                }
                ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Name oder Code: $c1, 193 …").desired_width(180.0));
            });
            ui.add_space(4.0);
            // Suche: Hexwert, Dezimalzahl oder Namensteil
            let q = self.search.trim().to_ascii_lowercase();
            let matches = |code: u8| -> bool {
                if q.is_empty() { return false; }
                if let Some(h) = q.strip_prefix('$') { return u8::from_str_radix(h, 16).map_or(false, |v| v == code); }
                if let Ok(n) = q.parse::<u16>() { return n == code as u16; }
                let (n, d) = describe(atari, 0, code);
                n.to_lowercase().contains(&q) || d.to_lowercase().contains(&q)
            };
            ui.horizontal_top(|ui| {
                // ---- Raster
                ui.vertical(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(36.0 + 16.0 * CELL, 18.0 + 16.0 * CELL), Sense::hover());
                    let (gx, gy) = (rect.left() + 36.0, rect.top() + 18.0);
                    let p = ui.painter_at(rect.expand(2.0));
                    for c in 0..16 { p.text(pos2(gx + c as f32 * CELL + CELL / 2.0, rect.top() + 8.0), Align2::CENTER_CENTER, format!("{c:X}"), egui::FontId::proportional(11.0), dim); }
                    for r in 0..16 { p.text(pos2(gx - 6.0, gy + r as f32 * CELL + CELL / 2.0), Align2::RIGHT_CENTER, format!("${r:X}_"), egui::FontId::proportional(11.0), dim); }
                    let grid = Rect::from_min_size(pos2(gx, gy), vec2(16.0 * CELL, 16.0 * CELL));
                    p.rect_filled(grid, 2.0, ui.visuals().extreme_bg_color);
                    let ink = ui.visuals().text_color();
                    let bg = ui.visuals().extreme_bg_color;
                    let tab = self.tab;
                    let mut clicked: Option<(u8, bool, bool)> = None;
                    for code in 0..=255u8 {
                        let (r, c) = ((code / 16) as f32, (code % 16) as f32);
                        let cell = Rect::from_min_size(pos2(gx + c * CELL, gy + r * CELL), vec2(CELL, CELL));
                        let resp = ui.interact(cell, ui.id().with(("cp", code)), Sense::click());
                        let in_basket = self.basket.contains(&code);
                        if code == self.sel { p.rect_filled(cell.shrink(1.0), 3.0, ui.visuals().selection.bg_fill); }
                        else if resp.hovered() { p.rect_filled(cell.shrink(1.0), 3.0, ui.visuals().widgets.hovered.bg_fill); }
                        match glyph_of(atari, tab, code) {
                            Some((i, inv)) => {
                                if let Some(t) = tex.as_ref().and_then(|t| t.get(i)) {
                                    let gr = Rect::from_min_size(cell.min + vec2(2.0, 2.0), vec2(8.0 * SCALE, 8.0 * SCALE));
                                    if inv && self.inverse { p.rect_filled(gr, 0.0, ink); Self::paint_glyph(ui, t, gr, bg); } else { Self::paint_glyph(ui, t, gr, ink); }
                                }
                            }
                            None => match if tab == 0 { abbr(atari, code) } else { None } {
                                Some(a) => { p.text(cell.center(), Align2::CENTER_CENTER, a, egui::FontId::proportional(9.0), accent); }
                                None => { p.text(cell.center(), Align2::CENTER_CENTER, "·", egui::FontId::proportional(12.0), dim); }
                            },
                        }
                        if in_basket { p.rect_stroke(cell.shrink(1.5), 3.0, Stroke::new(1.5, accent), egui::StrokeKind::Inside); }
                        if matches(code) { p.rect_stroke(cell.shrink(0.5), 3.0, Stroke::new(2.0, crate::theme::warn()), egui::StrokeKind::Inside); }
                        if resp.clicked() { clicked = Some((code, ui.input(|i| i.modifiers.shift), false)); }
                        if resp.double_clicked() { clicked = Some((code, false, true)); }
                        if resp.hovered() {
                            let (n, _) = describe(atari, tab, code);
                            egui::Tooltip::always_open(ui.ctx().clone(), ui.layer_id(), ui.id().with(("tip", code)), egui::PopupAnchor::Pointer).show(|ui| { ui.label(format!("${code:02X}  ({code})  {n}")); });
                        }
                    }
                    for k in 0..=16 {
                        let s = Stroke::new(0.5, ui.visuals().widgets.noninteractive.bg_stroke.color);
                        p.line_segment([pos2(gx + k as f32 * CELL, gy), pos2(gx + k as f32 * CELL, gy + 16.0 * CELL)], s);
                        p.line_segment([pos2(gx, gy + k as f32 * CELL), pos2(gx + 16.0 * CELL, gy + k as f32 * CELL)], s);
                    }
                    if let Some((code, shift, dbl)) = clicked {
                        if dbl {
                            // der erste Klick des Doppelklicks hat das Zeichen schon in die Auswahl gelegt
                            while self.basket.last() == Some(&code) { self.basket.pop(); }
                            self.sel = code;
                            let text = compose(atari, tab, self.mode, &[code]);
                            self.note_recent(code);
                            out = Out::Insert(text);
                        } else if shift && self.sel != code {
                            let (a, b) = (self.sel.min(code), self.sel.max(code));
                            for c in a..=b { if glyph_of(atari, tab, c).is_some() || tab == 0 { self.basket.push(c); } }
                            self.sel = code;
                        } else { self.sel = code; self.basket.push(code); }
                    }
                });
                // ---- Einzelheiten und Einfügeart
                ui.vertical(|ui| {
                    ui.set_width(250.0);
                    let sel = self.sel;
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_min_size(vec2(236.0, 150.0));
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(96.0, 96.0), Sense::hover());
                            ui.painter().rect_filled(r, 4.0, ui.visuals().extreme_bg_color);
                            if let (Some((i, inv)), Some(t)) = (glyph_of(atari, self.tab, sel), tex.as_ref()) {
                                if let Some(t) = t.get(i) {
                                    let gr = Rect::from_min_size(r.min + vec2(8.0, 8.0), vec2(80.0, 80.0));
                                    if inv && self.inverse { ui.painter().rect_filled(gr, 0.0, ui.visuals().text_color()); Self::paint_glyph(ui, t, gr, ui.visuals().extreme_bg_color); } else { Self::paint_glyph(ui, t, gr, ui.visuals().text_color()); }
                                }
                            }
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(format!("{} ${:02X}", tabs[self.tab], sel)).strong());
                                ui.label(format!("dezimal {sel}"));
                                match screen_value(atari, self.tab, sel) {
                                    Some(v) if self.tab == 0 => { ui.label(format!("{} ${v:02X}", if atari { "ANTIC-Code" } else { "Bildschirmcode" })); }
                                    _ => { ui.label(""); }
                                }
                            });
                        });
                        let (name, key) = describe(atari, self.tab, sel);
                        ui.label(egui::RichText::new(name).color(dim));
                        if !key.is_empty() { ui.label(egui::RichText::new(format!("Tastatur: {key}")).small().color(dim)); }
                        if literal_of(atari, self.tab, sel).is_none() { ui.label(egui::RichText::new("nicht direkt tippbar: in Zeichenketten als {…}").small().color(dim)); }
                    });
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Einfügen als").strong());
                    let ex = |m: Mode| compose(atari, self.tab, m, &[sel]);
                    for (m, name) in [(Mode::InString, "In Zeichenkette"), (Mode::Value, "Wert"), (Mode::ByteDir, "Byte-Direktive"), (Mode::Decimal, "Dezimalzahl"),
                                      (Mode::ScreenCode, if atari { "ANTIC-Code" } else { "Bildschirmcode" })] {
                        ui.horizontal(|ui| {
                            if ui.radio(self.mode == m, name).clicked() { self.mode = m; self.mode_user = true; }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(egui::RichText::new(ex(m)).monospace().color(dim)); });
                        });
                    }
                    ui.checkbox(&mut self.keep_open, "Dialog offen lassen (mehrere Zeichen)");
                    if self.mode_user && ui.small_button("Vorgabe nach Cursorposition").clicked() { self.mode_user = false; }
                });
            });
            ui.add_space(4.0);
            // ---- zuletzt benutzt
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Zuletzt benutzt").small().color(dim));
                for code in self.recent.clone() {
                    let (r, resp) = ui.allocate_exact_size(vec2(CELL, CELL), Sense::click());
                    ui.painter().rect_filled(r, 3.0, ui.visuals().extreme_bg_color);
                    if let (Some((i, _)), Some(t)) = (glyph_of(atari, self.tab, code), tex.as_ref()) { if let Some(t) = t.get(i) { Self::paint_glyph(ui, t, Rect::from_min_size(r.min + vec2(2.0, 2.0), vec2(24.0, 24.0)), ui.visuals().text_color()); } }
                    else if let Some(a) = abbr(atari, code) { ui.painter().text(r.center(), Align2::CENTER_CENTER, a, egui::FontId::proportional(9.0), accent); }
                    if resp.clicked() { self.sel = code; }
                }
            });
            // ---- Auswahlkorb
            crate::ui::widgets::card(ui, false, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Auswahl ({})", self.basket.len())).strong());
                    ui.label(egui::RichText::new("Klick fügt hinzu · Umschalt+Klick: Bereich · Entf: entfernen").small().color(dim));
                });
                ui.horizontal_wrapped(|ui| {
                    let mut remove = None;
                    for (i, c) in self.basket.iter().enumerate() {
                        let label = match abbr(atari, *c).filter(|_| self.tab == 0) { Some(a) => format!("${c:02x} {a}"), None => format!("${c:02x}") };
                        if ui.small_button(label).on_hover_text("entfernen").clicked() { remove = Some(i); }
                    }
                    if let Some(i) = remove { self.basket.remove(i); }
                });
                let preview = if self.basket.is_empty() { compose(atari, self.tab, self.mode, &[self.sel]) } else { compose(atari, self.tab, self.mode, &self.basket) };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(preview).monospace().color(crate::theme::accent()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if crate::shell::action_button(ui, "Einfügen", crate::shell::Kind::Primary, true).clicked() { out = self.insert_now(); }
                        if ui.button("Leeren").clicked() { self.basket.clear(); }
                    });
                });
            });
            ui.label(egui::RichText::new("Enter: einfügen · Pfeiltasten: wählen · Entf: letztes aus der Auswahl · Esc: schließen").small().color(dim));
            // Tastatur (nur wenn kein Textfeld den Fokus hat)
            if !ui.memory(|m| m.focused().is_some()) {
                let (enter, esc, del, l, r, u, d) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                    i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight), i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown)));
                if enter { out = self.insert_now(); }
                if esc { out = Out::Close; }
                if del { self.basket.pop(); }
                if l { self.sel = self.sel.wrapping_sub(1); }
                if r { self.sel = self.sel.wrapping_add(1); }
                if u { self.sel = self.sel.wrapping_sub(16); }
                if d { self.sel = self.sel.wrapping_add(16); }
            }
        });
        if !open { return Out::Close; }
        out
    }

    fn note_recent(&mut self, code: u8) {
        self.recent.retain(|c| *c != code);
        self.recent.insert(0, code);
        self.recent.truncate(8);
    }

    fn insert_now(&mut self) -> Out {
        let codes = if self.basket.is_empty() { vec![self.sel] } else { self.basket.clone() };
        let text = compose(self.atari, self.tab, self.mode, &codes);
        for c in &codes { self.note_recent(*c); }
        self.basket.clear();
        if text.is_empty() { return Out::None; }
        Out::Insert(text)
    }

    pub fn keep_open(&self) -> bool { self.keep_open }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_in_strings_uses_names_literals_and_repeats() {
        // PETSCII: CLR, Buchstaben (direkt: a = $41, A = $C1), ein Grafikzeichen als Hexwert, drei mal Cursor runter
        assert_eq!(compose(false, 0, Mode::InString, &[0x93, 0x41, 0xC1, 0xA1, 0x11, 0x11, 0x11]), "{clr}aA{$a1}{down*3}");
        assert_eq!(compose(false, 0, Mode::InString, &[0xC1 + 1, 0x31, 0x22]), "B1{$22}");
        assert_eq!(compose(true, 0, Mode::InString, &[0x7D, 0x41, 0x14, 0x9B]), "{clr}A{$14}{eol}");
        // Bildschirmcode-Reiter: nur Zahlen
        assert_eq!(compose(false, 1, Mode::InString, &[0x01, 0x41]), "{$01}{$41}");
    }

    #[test]
    fn compose_other_modes() {
        assert_eq!(compose(false, 0, Mode::Value, &[0xC1, 0x93]), "$c1, $93");
        assert_eq!(compose(false, 0, Mode::ByteDir, &[0xC1]), ".byte $c1");
        assert_eq!(compose(false, 0, Mode::Decimal, &[0xC1, 0x93]), "193, 147");
        assert_eq!(compose(false, 0, Mode::ScreenCode, &[0xC1, 0x93]), "$41");     // Steuerzeichen haben keinen Bildschirmcode
        assert_eq!(compose(true, 0, Mode::ScreenCode, &[0x14, 0x94]), "$54, $d4");
    }

    #[test]
    fn petscii_screen_mapping_matches_the_assembler() {
        // jedes PETSCII-Zeichen, das der Assembler aus einem ASCII-Buchstaben erzeugt, zeigt das Bildschirmcode-Zeichen von `.screen`
        assert_eq!(glyph_of(false, 0, 0x41).unwrap().0, 0x01);
        assert_eq!(glyph_of(false, 0, 0xC1).unwrap().0, 0x41);
        assert_eq!(glyph_of(false, 0, 0x93), None);
        assert_eq!(glyph_of(true, 0, 0x14).unwrap(), (0x54, false));
        assert_eq!(glyph_of(true, 0, 0x94).unwrap(), (0x54, true));
        assert_eq!(glyph_of(true, 0, 0x7D), None);
    }

    #[test]
    fn string_detection_for_the_default_mode() {
        let dir = std::env::temp_dir().join("vc-charpick-test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("t.asm"), "msg: .petscii \"AB\" ; \"x\"\n .byte \"q\"\n .text \"a\" ; Kommentar\n").unwrap();
        let mut d = crate::editor::Doc::open(&dir, "t.asm").unwrap();
        d.cursor = (1, 17);   // zwischen A und B
        assert!(d.in_text_string());
        d.cursor = (1, 22);   // hinter dem Semikolon
        assert!(!d.in_text_string());
        d.cursor = (2, 12);   // .byte ist keine Textdirektive
        assert!(!d.in_text_string());
        d.cursor = (3, 10);   // in .text "a"
        assert!(d.in_text_string());
        d.cursor = (3, 14);   // hinter der schließenden Anführung
        assert!(!d.in_text_string());
        assert_eq!(d.target_hint(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_composed_string_is_understood_by_the_assembler() {
        use vc_asm::tokens::{expand, Names};
        for code in 0..=255u8 {
            for (atari, names) in [(false, Names::Petscii), (true, Names::Atascii)] {
                let s = compose(atari, 0, Mode::InString, &[code, code, code]);
                let p = expand(&s, names).unwrap_or_else(|e| panic!("{s}: {e}"));
                assert_eq!(p.len(), 3, "{s}");
            }
        }
    }
}
