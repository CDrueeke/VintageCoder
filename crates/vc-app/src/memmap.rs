// SPDX-License-Identifier: GPL-3.0-or-later
//! Memory Map als Matrix (Tab „Memory Map“, Reiter „Matrix“): 16×16 Seiten zu je 256 Byte, gefärbt nach Art des
//! Speichers laut Bank-Preset des Ziels (`data/targets/*.target.json`, Abschnitt `banking`), darüber das Programm
//! aus der letzten Übersetzung (`.vcdbg`: Segmente, Bänke, Symbole). Der obere Teil kennt keine Oberfläche und ist getestet.
//! Es erscheinen nur echte Werte: ohne Bau steht dort „Bauen, um die Belegung zu sehen“, ohne Presets im Ziel nur die Segmente.
use crate::ui::tokens::t;
use crate::ui::widgets::{bitfield, chip, segmented};
use eframe::egui::{self, Color32, RichText};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vc_target::{BankPreset, Banking, Page, PageCategory, PageKind, Target};

// ---------------------------------------------------------------------------------------------
// Datenaufbereitung (ohne Oberfläche)
// ---------------------------------------------------------------------------------------------

/// Ein Segment des Programms (aus `.vcdbg`)
#[derive(Debug, Clone, PartialEq)]
pub struct Seg { pub start: u32, pub len: usize, pub bank: u32, pub name: String }

impl Seg {
    /// Letzte belegte Adresse (ein leeres Segment zählt als ein Byte)
    pub fn end(&self) -> u32 { self.start + self.len.max(1) as u32 - 1 }
}

/// Was die Matrix vom Programm weiß: Segmente und Symbole
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Facts { pub segs: Vec<Seg>, pub syms: Vec<(String, u32)> }

/// Liest Segmente und Symbole aus dem Text einer `.vcdbg`-Datei
pub fn parse_vcdbg(text: &str) -> Option<Facts> {
    let v: Value = serde_json::from_str(text).ok()?;
    let segs = v["segments"].as_array()?.iter().filter_map(|s| Some(Seg {
        start: s["start"].as_u64()? as u32, len: s["length"].as_u64()? as usize, bank: s["bank"].as_u64().unwrap_or(0) as u32, name: s["name"].as_str().unwrap_or("").into(),
    })).collect();
    let syms = v["symbols"].as_array().map(|a| a.iter().filter_map(|s| {
        let (n, val) = (s["name"].as_str()?, s["value"].as_i64()?);
        (0..0x10000).contains(&val).then(|| (n.to_string(), val as u32))
    }).collect()).unwrap_or_default();
    Some(Facts { segs, syms })
}

/// Belegte Bytes je Seite (nur Bank 0; überlappende Segmente zählen doppelt, höchstens 256)
pub fn page_usage(segs: &[Seg]) -> [u32; 256] {
    let mut u = [0u32; 256];
    for s in segs.iter().filter(|s| s.bank == 0) {
        let (a, b) = (s.start.min(0xFFFF), s.end().min(0xFFFF));
        for p in (a >> 8)..=(b >> 8) {
            let lo = a.max(p << 8);
            let hi = b.min((p << 8) + 255);
            u[p as usize] = (u[p as usize] + hi - lo + 1).min(256);
        }
    }
    u
}

/// Welche Bytes einer Seite das Programm belegt (Bank 0)
pub fn byte_usage(segs: &[Seg], page: u8) -> [bool; 256] {
    let mut u = [false; 256];
    let base = (page as u32) << 8;
    for s in segs.iter().filter(|s| s.bank == 0) {
        for b in s.start.max(base)..=s.end().min(base + 255) { if b >= base && b <= base + 255 { u[(b - base) as usize] = true; } }
    }
    u
}

/// Überlappungen von Segmenten derselben Bank: (Anfang, Ende) des gemeinsamen Bereichs
pub fn overlaps(segs: &[Seg]) -> Vec<(u32, u32)> {
    let mut v = vec![];
    for (i, a) in segs.iter().enumerate() {
        for b in &segs[i + 1..] {
            if a.bank != b.bank { continue; }
            let (lo, hi) = (a.start.max(b.start), a.end().min(b.end()));
            if lo <= hi { v.push((lo, hi)); }
        }
    }
    v
}

/// Quellstellen auf Seiten `first..=last`: (Datei, erste Zeile, letzte Zeile), je Datei ein Eintrag in Reihenfolge des Auftretens
pub fn page_sources(lines: &[(String, usize, u16, usize)], first: u8, last: u8) -> Vec<(String, usize, usize)> {
    let (lo, hi) = ((first as u32) << 8, ((last as u32) << 8) + 255);
    let mut out: Vec<(String, usize, usize)> = vec![];
    for (f, n, a, l) in lines {
        let (s, e) = (*a as u32, *a as u32 + (*l).max(1) as u32 - 1);
        if e < lo || s > hi { continue; }
        match out.iter_mut().find(|o| &o.0 == f) { Some(o) => { o.1 = o.1.min(*n); o.2 = o.2.max(*n); } None => out.push((f.clone(), *n, *n)) }
    }
    out
}

/// Symbole auf den Seiten `first..=last`, nach Adresse sortiert. Nur Symbole innerhalb eines Segments (Marken, keine Konstanten wie Chip-Register).
pub fn page_symbols(facts: &Facts, first: u8, last: u8) -> Vec<(String, u32)> {
    let (lo, hi) = ((first as u32) << 8, ((last as u32) << 8) + 255);
    let mut v: Vec<(String, u32)> = facts.syms.iter()
        .filter(|(n, a)| *a >= lo && *a <= hi && !n.contains(".__m") && facts.segs.iter().any(|s| s.bank == 0 && *a >= s.start && *a <= s.end()))
        .cloned().collect();
    v.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    v
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level { Ok, Info, Warn, Err }

#[derive(Debug, Clone, PartialEq)]
pub struct Check { pub level: Level, pub text: String }

fn range_text(a: u32, b: u32) -> String { format!("${a:04X}–${b:04X}") }

/// Prüfungen: Überlappung (Fehler), Stack (Warnung), Zeropage und Seiten unter ROM/E/A, geteilte Seiten (Hinweise)
pub fn checks(facts: &Facts, plan: Option<&[Page; 256]>) -> Vec<Check> {
    let mut v = vec![];
    let ov = overlaps(&facts.segs);
    if ov.is_empty() { v.push(Check { level: Level::Ok, text: "0 Überlappungen".into() }); }
    else { v.push(Check { level: Level::Err, text: format!("{} Überlappung(en): {}", ov.len(), ov.iter().take(3).map(|(a, b)| range_text(*a, *b)).collect::<Vec<_>>().join(", ")) }); }
    let usage = page_usage(&facts.segs);
    if usage[1] > 0 { v.push(Check { level: Level::Warn, text: format!("Programm belegt {} Byte im Stack (Seite $01)", usage[1]) }); }
    else { v.push(Check { level: Level::Ok, text: "Stack frei (Seite $01)".into() }); }
    if usage[0] > 0 { v.push(Check { level: Level::Info, text: format!("Zeropage: {} Byte belegt", usage[0]) }); }
    let shared: Vec<u8> = (0..=255u8).filter(|&p| facts.segs.iter().filter(|s| s.bank == 0 && (s.start >> 8) <= p as u32 && (s.end().min(0xFFFF) >> 8) >= p as u32).count() > 1).collect();
    if !shared.is_empty() { v.push(Check { level: Level::Info, text: format!("{} Seite(n) von mehreren Segmenten genutzt, z. B. ${:02X}", shared.len(), shared[0]) }); }
    if let Some(plan) = plan {
        let hidden: Vec<u8> = (0..=255u8).filter(|&p| usage[p as usize] > 0 && plan[p as usize].kind != PageKind::Ram).collect();
        if !hidden.is_empty() { v.push(Check { level: Level::Info, text: format!("{} Seite(n) des Programms liegen in dieser Belegung unter ROM oder E/A, z. B. ${:02X}", hidden.len(), hidden[0]) }); }
    }
    v
}

/// Ansicht der Matrix
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View { Pages, Blocks, Bytes }

impl View {
    fn parse(s: &str) -> View { match s { "blocks" => View::Blocks, "bytes" => View::Bytes, _ => View::Pages } }
}

/// Filter nach Art
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter { All, Ram, Rom, Io, Program }

impl Filter {
    fn parse(s: &str) -> Filter { match s { "ram" => Filter::Ram, "rom" => Filter::Rom, "io" => Filter::Io, "prog" => Filter::Program, _ => Filter::All } }
    /// Passt die Seite (Art, belegte Bytes) zum Filter?
    pub fn matches(self, kind: PageKind, used: u32) -> bool {
        match self {
            Filter::All => true,
            Filter::Ram => kind == PageKind::Ram,
            Filter::Rom => matches!(kind, PageKind::Rom | PageKind::BasicRom | PageKind::KernalRom | PageKind::CharRom | PageKind::Module),
            Filter::Io => kind == PageKind::Io,
            Filter::Program => used > 0,
        }
    }
}

/// Seitenplan, wenn es keine Presets gibt: alles RAM, Seite 0 und 1 als Zeropage und Stack
pub fn plain_plan() -> [Page; 256] {
    let mut p = [Page { kind: PageKind::Ram, category: PageCategory::Free }; 256];
    for (i, x) in p.iter_mut().enumerate() { x.category = PageCategory::default_for(PageKind::Ram, i as u8); }
    p
}

pub fn kind_label(k: PageKind) -> &'static str {
    match k { PageKind::Ram => "RAM", PageKind::Rom => "ROM", PageKind::BasicRom => "BASIC-ROM", PageKind::KernalRom => "System-ROM (KERNAL/OS)", PageKind::CharRom => "Zeichen-ROM", PageKind::Io => "E/A", PageKind::Module => "Modul" }
}

pub fn category_label(c: PageCategory) -> &'static str {
    match c {
        PageCategory::Zeropage => "Zeropage", PageCategory::Stack => "Stack", PageCategory::Screen => "Bildschirm", PageCategory::Sprites => "Sprites", PageCategory::Charset => "Zeichensatz",
        PageCategory::Music => "Musik", PageCategory::Io => "E/A", PageCategory::Rom => "ROM", PageCategory::Basic => "BASIC", PageCategory::Free => "frei (RAM)",
    }
}

/// Kategorien, die im Seitenplan vorkommen, in fester Reihenfolge (für die Legende)
pub fn legend(plan: &[Page; 256]) -> Vec<PageCategory> {
    use PageCategory::*;
    [Zeropage, Stack, Screen, Sprites, Charset, Music, Basic, Rom, Io, Free].into_iter().filter(|c| plan.iter().any(|p| p.category == *c)).collect()
}

/// Presets der Liste als (Kennung, Beschriftung); Presets nur für ein Modell tragen „nur 130XE“ o. ä. im Namen
pub fn preset_label(p: &BankPreset) -> String {
    let m = if p.models.len() == 1 { format!(" – nur {}", p.models[0].to_uppercase()) } else { String::new() };
    format!("${:02X}  {}{}", p.value, p.name, m)
}

// ---------------------------------------------------------------------------------------------
// Oberfläche
// ---------------------------------------------------------------------------------------------

/// Eingaben aus der Anwendung (Bau-Ergebnis, Datenordner, Ziel)
pub struct Input<'a> {
    pub data: Option<&'a Path>,
    pub target: &'a str,
    /// Artefakt des letzten Baus (daneben liegt die `.vcdbg`); `None` = noch nicht gebaut
    pub artifact: Option<&'a Path>,
    pub segments: &'a [(u32, usize)],
    pub symbols: Option<&'a HashMap<String, i64>>,
    pub lines: &'a [(String, usize, u16, usize)],
}

#[derive(Clone)]
struct State { view: String, filter: String, sel: u8, preset: Option<String>, value: u8, init: bool }

impl Default for State {
    fn default() -> Self { State { view: "pages".into(), filter: "all".into(), sel: 0, preset: None, value: 0, init: false } }
}

/// Zusammengehörige Daten eines Ziels (einmal geladen)
struct Loaded { id: String, target: Option<Arc<Target>> }

struct FactsCache { key: (PathBuf, u64, Option<std::time::SystemTime>), facts: Arc<Facts> }

fn light() -> bool { crate::theme::is_light() }
fn rgb(dark: (u8, u8, u8), lightc: (u8, u8, u8)) -> Color32 { let (r, g, b) = if light() { lightc } else { dark }; Color32::from_rgb(r, g, b) }

/// Farbe je Kategorie (Gelb und Rot bleiben echten Fehlern vorbehalten)
fn category_color(c: PageCategory) -> Color32 {
    let k = t();
    match c {
        PageCategory::Free => k.bg2,
        PageCategory::Zeropage => rgb((0x2a, 0x8f, 0xb4), (0x7f, 0xc4, 0xdf)),
        PageCategory::Stack => rgb((0x1f, 0x9d, 0x8f), (0x72, 0xcc, 0xc0)),
        PageCategory::Screen => rgb((0xb4, 0x4f, 0x8a), (0xe0, 0x9c, 0xc0)),
        PageCategory::Sprites => rgb((0xb8, 0x55, 0x5f), (0xe3, 0xa0, 0xa6)),
        PageCategory::Charset => rgb((0x2f, 0x8f, 0x5a), (0x8f, 0xd0, 0xa8)),
        PageCategory::Music => rgb((0x9a, 0x86, 0x2b), (0xd6, 0xc4, 0x78)),
        PageCategory::Io => rgb((0xb0, 0x6c, 0x2e), (0xe0, 0xb0, 0x80)),
        PageCategory::Rom => rgb((0x6b, 0x66, 0x7c), (0xb4, 0xb0, 0xc4)),
        PageCategory::Basic => rgb((0x7a, 0x4c, 0xb8), (0xbd, 0x9c, 0xe0)),
    }
}

fn text_on(bg: Color32) -> Color32 {
    let l = 0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
    if l > 140.0 { Color32::from_rgb(0x10, 0x14, 0x1c) } else { Color32::from_rgb(0xf0, 0xf3, 0xf8) }
}

/// Reiter „Matrix“ / „Liste“. Liefert `true` für die Matrix.
pub fn tabs(ui: &mut egui::Ui) -> bool {
    let id = egui::Id::new("memmap_tab");
    let mut cur: String = ui.data(|d| d.get_temp(id)).unwrap_or_else(|| "matrix".into());
    if segmented(ui, &mut cur, &[("matrix", "Matrix"), ("list", "Liste")]) { ui.data_mut(|d| d.insert_temp(id, cur.clone())); }
    ui.add_space(6.0);
    cur == "matrix"
}

fn load_target(ui: &egui::Ui, input: &Input) -> Option<Arc<Target>> {
    let id = egui::Id::new("memmap_target");
    let cached: Option<Arc<Loaded>> = ui.data(|d| d.get_temp(id));
    if let Some(c) = cached { if c.id == input.target { return c.target.clone(); } }
    let target = input.data.and_then(|d| vc_target::load_target(d, input.target).ok()).map(Arc::new);
    ui.data_mut(|d| d.insert_temp(id, Arc::new(Loaded { id: input.target.to_string(), target: target.clone() })));
    target
}

fn load_facts(ui: &egui::Ui, input: &Input) -> Option<Arc<Facts>> {
    let art = input.artifact?;
    let path = art.with_extension("vcdbg");
    let meta = std::fs::metadata(&path).ok();
    let key = (path.clone(), meta.as_ref().map_or(0, |m| m.len()), meta.as_ref().and_then(|m| m.modified().ok()));
    let id = egui::Id::new("memmap_facts");
    let cached: Option<Arc<FactsCache>> = ui.data(|d| d.get_temp(id));
    if let Some(c) = cached { if c.key == key { return Some(c.facts.clone()); } }
    let facts = std::fs::read_to_string(&path).ok().and_then(|t| parse_vcdbg(&t)).unwrap_or_else(|| Facts {
        segs: input.segments.iter().map(|(a, l)| Seg { start: *a, len: *l, bank: 0, name: String::new() }).collect(),
        syms: input.symbols.map(|m| m.iter().filter(|(_, v)| (0..0x10000).contains(*v)).map(|(n, v)| (n.clone(), *v as u32)).collect()).unwrap_or_default(),
    });
    let facts = Arc::new(facts);
    ui.data_mut(|d| d.insert_temp(id, Arc::new(FactsCache { key, facts: facts.clone() })));
    Some(facts)
}

pub fn show(ui: &mut egui::Ui, input: &Input) {
    let sid = egui::Id::new("memmap_state");
    let mut st: State = ui.data(|d| d.get_temp(sid)).unwrap_or_default();
    let target = load_target(ui, input);
    let facts = load_facts(ui, input);
    let banking: Option<&Banking> = target.as_ref().and_then(|t| t.banking.as_ref());
    // Erster Aufruf (oder Zielwechsel): Standard-Preset, Auswahl auf die erste belegte Seite
    let tkey = egui::Id::new(("memmap_for", input.target));
    if !st.init || ui.data(|d| d.get_temp::<bool>(tkey)).is_none() {
        ui.data_mut(|d| d.insert_temp(tkey, true));
        st.init = true;
        if let Some(p) = banking.and_then(|b| b.default_preset()) { st.preset = Some(p.id.clone()); st.value = p.value; } else { st.preset = None; }
        st.sel = facts.as_ref().and_then(|f| page_usage(&f.segs).iter().position(|u| *u > 0)).unwrap_or(0) as u8;
        if std::env::var("VC_SHOT").is_ok() {
            if let Ok(v) = std::env::var("VC_MM_VIEW") { st.view = v; }
            if let Ok(p) = std::env::var("VC_MM_PRESET") { if let Some(pr) = banking.and_then(|b| b.preset(&p)) { st.preset = Some(pr.id.clone()); st.value = pr.value; } }
            if let Some(p) = std::env::var("VC_MM_PAGE").ok().and_then(|p| u8::from_str_radix(p.trim_start_matches('$'), 16).ok()) { st.sel = p; }
        }
    }
    let k = t();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let tname = target.as_ref().map_or(input.target.to_string(), |t| t.id.clone());
        // Kopf: Ziel, Ansicht, Filter
        ui.horizontal_wrapped(|ui| {
            chip(ui, &format!("{} · 64 KB", tname.to_uppercase()), k.cy);
            segmented(ui, &mut st.view, &[("pages", "256 Seiten"), ("blocks", "4-KB-Block"), ("bytes", "Byte")]);
        });
        ui.horizontal_wrapped(|ui| {
            segmented(ui, &mut st.filter, &[("all", "Alle"), ("prog", "Programm"), ("ram", "RAM"), ("rom", "ROM"), ("io", "E/A")]);
        });
        ui.add_space(4.0);
        // Bank-Umschaltung
        let mut plan: [Page; 256] = plain_plan();
        let mut preset_note: Option<String> = None;
        match banking {
            Some(b) => {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(b.register.name.as_str()).small().color(k.mute));
                    ui.label(RichText::new(format!("${:04X}", b.register.addr)).small().monospace().color(k.dim));
                    let bits: Vec<(u8, &str, &str)> = b.register.bits.iter().map(|x| (x.bit, x.name.as_str(), x.hint.as_str())).collect();
                    if bitfield(ui, &mut st.value, &bits) { st.preset = b.preset_for_value(st.value).map(|p| p.id.clone()); }
                });
                ui.horizontal(|ui| {   // Preset in eigener Zeile: ragt so auch bei vielen Bits nicht über den Rand
                    let cur = st.preset.as_deref().and_then(|i| b.preset(i)).map_or_else(|| format!("${:02X}  (kein Preset)", st.value), preset_label);
                    egui::ComboBox::from_id_salt("memmap_preset").selected_text(cur).width((ui.available_width() - 24.0).clamp(120.0, 360.0)).show_ui(ui, |ui| {
                        for p in &b.presets {
                            if ui.selectable_label(st.preset.as_deref() == Some(p.id.as_str()), preset_label(p)).on_hover_text(&p.note).clicked() { st.preset = Some(p.id.clone()); st.value = p.value; }
                        }
                    });
                });
                match st.preset.as_deref().and_then(|i| b.preset(i)).or_else(|| b.default_preset()) {
                    Some(p) => {
                        plan = b.page_plan(p);
                        if st.preset.is_none() { preset_note = Some(format!("Für ${:02X} ist keine Belegung hinterlegt; gezeigt wird das Standard-Preset ${:02X}.", st.value, p.value)); }
                        else if !p.note.is_empty() { preset_note = Some(p.note.clone()); }
                    }
                    None => {}
                }
            }
            None => { ui.label(RichText::new("Keine Bank-Presets hinterlegt: die Matrix zeigt nur die Segmente des Programms.").small().color(k.mute)); }
        }
        if let Some(n) = &preset_note { ui.label(RichText::new(n.as_str()).small().color(k.mute)); }
        ui.add_space(4.0);

        let usage = facts.as_ref().map(|f| page_usage(&f.segs)).unwrap_or([0; 256]);
        let filter = Filter::parse(&st.filter);
        let view = View::parse(&st.view);
        match view {
            View::Pages => grid_pages(ui, &plan, &usage, filter, &mut st),
            View::Blocks => grid_blocks(ui, &plan, &usage, filter, &mut st),
            View::Bytes => grid_bytes(ui, &plan, facts.as_deref(), &mut st),
        }
        ui.add_space(4.0);
        // Legende
        ui.horizontal_wrapped(|ui| {
            for c in legend(&plan) { legend_item(ui, category_color(c), category_label(c)); }
            if facts.is_some() { legend_item(ui, k.cy, "Programm"); }
        });
        ui.add_space(6.0);
        inspector(ui, input, &plan, facts.as_deref(), &usage, view, &st);
    });
    ui.data_mut(|d| d.insert_temp(sid, st));
}

fn legend_item(ui: &mut egui::Ui, color: Color32, text: &str) {
    let k = t();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().rect_filled(r, 2.0, color);
        ui.painter().rect_stroke(r, 2.0, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside);   // helle Farben (frei) bleiben auf hellem Grund sichtbar
        ui.label(RichText::new(text).small().color(k.mute));
    });
    ui.add_space(4.0);
}

/// Größe einer Zelle bei `n` Spalten in der verfügbaren Breite (links Platz für die Zeilenbeschriftung)
fn cell_size(ui: &egui::Ui, n: usize, head: f32) -> f32 { ((ui.available_width() - head - 4.0) / n as f32).clamp(14.0, 40.0) }

fn draw_cell(ui: &egui::Ui, rect: egui::Rect, plan: &[Page], used: &[u32], dim: bool, selected: bool, label: Option<&str>) {
    // plan/used: die Seiten der Zelle (eine bei „256 Seiten“, 16 bei „4-KB-Block“)
    let k = t();
    let p = ui.painter();
    let n = plan.len().max(1);
    let w = rect.width() / n as f32;
    for (i, pg) in plan.iter().enumerate() {
        let col = category_color(pg.category);
        let r = egui::Rect::from_min_size(egui::pos2(rect.left() + w * i as f32, rect.top()), egui::vec2(w + 0.5, rect.height()));
        p.rect_filled(r, 0.0, if dim { col.gamma_multiply(0.30) } else { col });
        let frac = used[i].min(256) as f32 / 256.0;
        if frac > 0.0 {
            let h = (rect.height() * frac).max(2.0).min(rect.height());
            let o = egui::Rect::from_min_max(egui::pos2(r.left(), rect.bottom() - h), r.right_bottom());
            p.rect_filled(o, 0.0, if dim { k.cy.gamma_multiply(0.35) } else { k.cy });
        }
    }
    p.rect_stroke(rect, 2.0, egui::Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { k.fg } else { k.line }), egui::StrokeKind::Inside);
    if let Some(l) = label {
        let full = used.iter().all(|u| *u >= 256);
        let bg = if full { k.cy } else { category_color(plan[0].category) };
        let c = if dim { text_on(bg).gamma_multiply(0.5) } else { text_on(bg) };
        p.text(rect.center(), egui::Align2::CENTER_CENTER, l, egui::FontId::monospace((rect.width() * 0.38).clamp(8.0, 12.0)), c);
    }
}

fn page_tip(p: u8, pg: Page, used: u32) -> String {
    let mut s = format!("Seite ${p:02X} · ${:04X}–${:04X}\n{} · {}", (p as u32) << 8, ((p as u32) << 8) + 255, kind_label(pg.kind), category_label(pg.category));
    if used > 0 { s.push_str(&format!("\nProgramm: {used} von 256 Byte")); }
    s
}

fn grid_pages(ui: &mut egui::Ui, plan: &[Page; 256], usage: &[u32; 256], filter: Filter, st: &mut State) {
    let k = t();
    let head = 44.0;
    let cs = cell_size(ui, 16, head);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(head + cs * 16.0, 16.0 + cs * 16.0), egui::Sense::hover());
    let mono = egui::FontId::monospace(9.0);
    for c in 0..16 { ui.painter().text(egui::pos2(rect.left() + head + cs * (c as f32 + 0.5), rect.top() + 8.0), egui::Align2::CENTER_CENTER, format!("+{c:X}"), mono.clone(), k.dim); }
    for r in 0..16 {
        let y = rect.top() + 16.0 + cs * r as f32;
        ui.painter().text(egui::pos2(rect.left() + head - 4.0, y + cs * 0.5), egui::Align2::RIGHT_CENTER, format!("${r:X}000"), mono.clone(), k.dim);
        for c in 0..16 {
            let p = (r * 16 + c) as u8;
            let cr = egui::Rect::from_min_size(egui::pos2(rect.left() + head + cs * c as f32, y), egui::vec2(cs - 1.0, cs - 1.0));
            let (pg, u) = (plan[p as usize], usage[p as usize]);
            let dim = !filter.matches(pg.kind, u);
            let resp = ui.interact(cr, ui.id().with(("mmcell", p)), egui::Sense::click());
            let label = format!("{p:02X}");
            draw_cell(ui, cr, &[pg], &[u], dim, st.sel == p, if cs >= 20.0 { Some(label.as_str()) } else { None });
            if resp.clicked() { st.sel = p; }
            resp.on_hover_text(page_tip(p, pg, u));
        }
    }
}

fn grid_blocks(ui: &mut egui::Ui, plan: &[Page; 256], usage: &[u32; 256], filter: Filter, st: &mut State) {
    let cs = ((ui.available_width() - 4.0) / 4.0).clamp(48.0, 120.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(cs * 4.0, cs * 0.62 * 4.0), egui::Sense::hover());
    let ch = cs * 0.62;
    for b in 0..16usize {
        let (col, row) = (b % 4, b / 4);
        let cr = egui::Rect::from_min_size(egui::pos2(rect.left() + cs * col as f32, rect.top() + ch * row as f32), egui::vec2(cs - 3.0, ch - 3.0));
        let (pl, us) = (&plan[b * 16..b * 16 + 16], &usage[b * 16..b * 16 + 16]);
        let dim = !(0..16).any(|i| filter.matches(pl[i].kind, us[i]));
        let resp = ui.interact(cr, ui.id().with(("mmblock", b)), egui::Sense::click());
        draw_cell(ui, cr, pl, us, dim, (st.sel as usize) / 16 == b, None);
        let used: u32 = us.iter().sum();
        let label = format!("${:X}000", b);
        ui.painter().text(cr.left_top() + egui::vec2(5.0, 3.0), egui::Align2::LEFT_TOP, label, egui::FontId::monospace(11.0), Color32::from_rgb(0xf0, 0xf3, 0xf8));
        if used > 0 { ui.painter().text(cr.right_bottom() - egui::vec2(5.0, 3.0), egui::Align2::RIGHT_BOTTOM, format!("{:.0} %", used as f32 / 4096.0 * 100.0), egui::FontId::monospace(10.0), Color32::from_rgb(0xf0, 0xf3, 0xf8)); }
        if resp.clicked() { st.sel = (b * 16) as u8; }
        resp.on_hover_text(format!("${:04X}–${:04X}\nProgramm: {used} von 4096 Byte", b * 0x1000, b * 0x1000 + 0xFFF));
    }
}

fn grid_bytes(ui: &mut egui::Ui, plan: &[Page; 256], facts: Option<&Facts>, st: &mut State) {
    let k = t();
    let p = st.sel;
    ui.horizontal(|ui| {
        if ui.small_button("◀").clicked() { st.sel = st.sel.wrapping_sub(1); }
        ui.label(RichText::new(format!("Seite ${p:02X}  ${:04X}–${:04X}", (p as u32) << 8, ((p as u32) << 8) + 255)).monospace());
        if ui.small_button("▶").clicked() { st.sel = st.sel.wrapping_add(1); }
    });
    let used = facts.map(|f| byte_usage(&f.segs, p)).unwrap_or([false; 256]);
    let head = 36.0;
    let cs = cell_size(ui, 16, head);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(head + cs * 16.0, 16.0 + cs * 16.0), egui::Sense::hover());
    let mono = egui::FontId::monospace(9.0);
    for c in 0..16 { ui.painter().text(egui::pos2(rect.left() + head + cs * (c as f32 + 0.5), rect.top() + 8.0), egui::Align2::CENTER_CENTER, format!("+{c:X}"), mono.clone(), k.dim); }
    let pg = plan[p as usize];
    for r in 0..16 {
        let y = rect.top() + 16.0 + cs * r as f32;
        ui.painter().text(egui::pos2(rect.left() + head - 4.0, y + cs * 0.5), egui::Align2::RIGHT_CENTER, format!("+${r:X}0"), mono.clone(), k.dim);
        for c in 0..16 {
            let i = r * 16 + c;
            let cr = egui::Rect::from_min_size(egui::pos2(rect.left() + head + cs * c as f32, y), egui::vec2(cs - 1.0, cs - 1.0));
            let col = if used[i] { k.cy } else { category_color(pg.category) };
            ui.painter().rect_filled(cr, 2.0, col);
            ui.painter().rect_stroke(cr, 2.0, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside);
            if cs >= 18.0 { ui.painter().text(cr.center(), egui::Align2::CENTER_CENTER, format!("{i:02X}"), egui::FontId::monospace(9.0), text_on(col)); }
            let resp = ui.interact(cr, ui.id().with(("mmbyte", i)), egui::Sense::hover());
            resp.on_hover_text(format!("${:04X} · {} · {}", ((p as u32) << 8) + i as u32, kind_label(pg.kind), if used[i] { "vom Programm belegt" } else { "frei" }));
        }
    }
}

fn kv(ui: &mut egui::Ui, key: &str, value: impl Into<egui::WidgetText>) {
    ui.horizontal_wrapped(|ui| {
        ui.add_sized([70.0, 18.0], egui::Label::new(RichText::new(key).small().color(t().dim)));
        ui.label(value);
    });
}

fn inspector(ui: &mut egui::Ui, input: &Input, plan: &[Page; 256], facts: Option<&Facts>, usage: &[u32; 256], view: View, st: &State) {
    let k = t();
    let (first, last) = if view == View::Blocks { ((st.sel & 0xF0), (st.sel | 0x0F)) } else { (st.sel, st.sel) };
    let (lo, hi) = ((first as u32) << 8, ((last as u32) << 8) + 255);
    egui::Frame::new().fill(k.bg1).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new("Segment-Inspektor").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(if first == last { format!("Seite ${first:02X}") } else { format!("Seiten ${first:02X}–${last:02X}") }).monospace().color(k.mute));
            });
        });
        kv(ui, "Adresse", RichText::new(range_text(lo, hi)).monospace());
        let pg = plan[first as usize];
        let kinds: Vec<&str> = { let mut v: Vec<&str> = vec![]; for p in first..=last { let l = kind_label(plan[p as usize].kind); if !v.contains(&l) { v.push(l); } } v };
        kv(ui, "Art", format!("{} · {}", kinds.join(", "), if first == last { category_label(pg.category) } else { "gemischt" }));
        let Some(f) = facts else {
            ui.add_space(4.0);
            ui.label(RichText::new("Bauen, um die Belegung zu sehen.").color(k.mute));
            return;
        };
        let used: u32 = (first..=last).map(|p| usage[p as usize]).sum();
        let total = (last as u32 - first as u32 + 1) * 256;
        kv(ui, "Belegung", RichText::new(format!("{:.0} % Programm ({used} von {total} Byte)", used as f32 / total as f32 * 100.0)).color(if used > 0 { k.cy } else { k.mute }));
        let segs: Vec<&Seg> = f.segs.iter().filter(|s| s.end() >= lo && s.start <= hi).collect();
        if segs.is_empty() { kv(ui, "Segment", RichText::new("keines").color(k.mute)); }
        for s in segs.iter().take(6) {
            kv(ui, "Segment", RichText::new(format!("{}{} {}", if s.name.is_empty() { "" } else { &s.name }, if s.name.is_empty() { "" } else { " ·" }, range_text(s.start, s.end())) + &if s.bank != 0 { format!(" · Bank {}", s.bank) } else { String::new() }).monospace());
        }
        if segs.len() > 6 { kv(ui, "", RichText::new(format!("… und {} weitere", segs.len() - 6)).small().color(k.dim)); }
        let src = page_sources(input.lines, first, last);
        if src.is_empty() { kv(ui, "Quelle", RichText::new("–").color(k.mute)); }
        for (file, a, b) in src.iter().take(4) { kv(ui, "Quelle", RichText::new(if a == b { format!("{file}:{a}") } else { format!("{file}:{a}–{b}") }).monospace()); }
        let syms = page_symbols(f, first, last);
        ui.add_space(4.0);
        ui.label(RichText::new(format!("Symbole auf dieser Seite ({})", syms.len())).small().strong());
        for (n, a) in syms.iter().take(12) {
            ui.horizontal(|ui| { ui.label(RichText::new(format!("${a:04X}")).monospace().small().color(k.cy)); ui.label(RichText::new(n.as_str()).monospace().small()); });
        }
        if syms.len() > 12 { ui.label(RichText::new(format!("… und {} weitere", syms.len() - 12)).small().color(k.dim)); }
        ui.add_space(4.0);
        ui.label(RichText::new("Prüfungen").small().strong());
        ui.horizontal_wrapped(|ui| {
            for c in checks(f, Some(plan)) {
                let col = match c.level { Level::Ok => k.gr, Level::Info => k.mute, Level::Warn => k.am, Level::Err => k.rd };
                chip(ui, &c.text, col);
            }
        });
        let other = f.segs.iter().filter(|s| s.bank != 0).count();
        if other > 0 { ui.label(RichText::new(format!("{other} Segment(e) in Bänken ungleich 0 sind nicht in der Matrix.")).small().color(k.dim)); }
    });
}

// ---------------------------------------------------------------------------------------------
// Tests der Datenaufbereitung
// ---------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start: u32, len: usize, bank: u32) -> Seg { Seg { start, len, bank, name: "Main".into() } }

    #[test]
    fn vcdbg_wird_gelesen() {
        let t = r#"{"format":1,"symbols":[{"name":"start","value":2064},{"name":"neg","value":-1},{"name":"hoch","value":70000}],
            "segments":[{"start":2049,"length":12,"bank":0,"name":"Main"},{"start":32768,"length":4,"bank":2,"name":"Rom"}],"lines":[]}"#;
        let f = parse_vcdbg(t).unwrap();
        assert_eq!(f.segs.len(), 2);
        assert_eq!(f.segs[1], Seg { start: 32768, len: 4, bank: 2, name: "Rom".into() });
        assert_eq!(f.syms, vec![("start".to_string(), 2064)]);
        assert!(parse_vcdbg("kein json").is_none());
    }

    #[test]
    fn belegung_je_seite() {
        // $07FE–$0802 liegt auf zwei Seiten: 2 Byte auf $07, 3 Byte auf $08
        let u = page_usage(&[seg(0x07FE, 5, 0)]);
        assert_eq!((u[7], u[8], u[6], u[9]), (2, 3, 0, 0));
        // Bank 1 zählt nicht, ein volles Segment füllt die Seiten ganz
        let u = page_usage(&[seg(0x2000, 0x200, 0), seg(0x4000, 100, 1)]);
        assert_eq!((u[0x20], u[0x21], u[0x22], u[0x40]), (256, 256, 0, 0));
        // Segment bis $FFFF
        assert_eq!(page_usage(&[seg(0xFFF0, 16, 0)])[0xFF], 16);
    }

    #[test]
    fn belegung_je_byte() {
        let b = byte_usage(&[seg(0x0805, 3, 0)], 8);
        assert!(!b[4] && b[5] && b[6] && b[7] && !b[8]);
        assert!(byte_usage(&[seg(0x0805, 3, 0)], 9).iter().all(|x| !x));
    }

    #[test]
    fn ueberlappung_nur_in_derselben_bank() {
        assert_eq!(overlaps(&[seg(0x1000, 0x100, 0), seg(0x10F0, 0x20, 0)]), vec![(0x10F0, 0x10FF)]);
        assert!(overlaps(&[seg(0x1000, 0x100, 0), seg(0x10F0, 0x20, 1)]).is_empty());
        assert!(overlaps(&[seg(0x1000, 0x100, 0), seg(0x1100, 0x20, 0)]).is_empty());
    }

    #[test]
    fn quellen_und_symbole_der_seite() {
        let lines = vec![("a.asm".to_string(), 3, 0x0800u16, 3), ("a.asm".to_string(), 4, 0x0803, 2), ("b.asm".to_string(), 9, 0x0900, 1), ("c.asm".to_string(), 1, 0x2000, 1)];
        assert_eq!(page_sources(&lines, 8, 8), vec![("a.asm".to_string(), 3, 4)]);
        assert_eq!(page_sources(&lines, 8, 9).len(), 2);
        assert!(page_sources(&lines, 0x30, 0x30).is_empty());
        let f = Facts { segs: vec![seg(0x0801, 0x20, 0)], syms: vec![("main".into(), 0x0810), ("SPRITE_PTRS".into(), 0x07F8), ("VIC.__m1".into(), 0x0812), ("init".into(), 0x0805)] };
        // Konstante außerhalb des Segments und interne Makro-Marken fallen weg
        assert_eq!(page_symbols(&f, 8, 8), vec![("init".to_string(), 0x0805), ("main".to_string(), 0x0810)]);
    }

    #[test]
    fn pruefungen() {
        let ok = Facts { segs: vec![seg(0x0801, 0x20, 0)], syms: vec![] };
        let c = checks(&ok, None);
        assert_eq!(c[0], Check { level: Level::Ok, text: "0 Überlappungen".into() });
        assert!(c.iter().any(|x| x.level == Level::Ok && x.text.contains("Stack frei")));
        let bad = Facts { segs: vec![seg(0x0100, 0x20, 0), seg(0x0110, 0x20, 0), seg(0x0010, 4, 0)], syms: vec![] };
        let c = checks(&bad, None);
        assert_eq!(c[0].level, Level::Err);
        assert!(c[0].text.contains("$0110–$011F"));
        assert!(c.iter().any(|x| x.level == Level::Warn && x.text.contains("Stack")));
        assert!(c.iter().any(|x| x.level == Level::Info && x.text.contains("Zeropage: 4")));
        assert!(c.iter().any(|x| x.text.contains("mehreren Segmenten")));
    }

    #[test]
    fn pruefung_unter_rom_nutzt_den_seitenplan() {
        let mut plan = plain_plan();
        for p in 0xA0..=0xBF { plan[p] = Page { kind: PageKind::BasicRom, category: PageCategory::Basic }; }
        let f = Facts { segs: vec![seg(0xA000, 0x100, 0), seg(0x0801, 4, 0)], syms: vec![] };
        let c = checks(&f, Some(&plan));
        assert!(c.iter().any(|x| x.text.contains("unter ROM oder E/A, z. B. $A0")));
        assert!(!checks(&f, None).iter().any(|x| x.text.contains("unter ROM")));
    }

    #[test]
    fn filter_nach_art() {
        assert!(Filter::All.matches(PageKind::Io, 0));
        assert!(Filter::Ram.matches(PageKind::Ram, 0) && !Filter::Ram.matches(PageKind::Io, 0));
        assert!(Filter::Rom.matches(PageKind::CharRom, 0) && Filter::Rom.matches(PageKind::BasicRom, 0) && !Filter::Rom.matches(PageKind::Ram, 0));
        assert!(Filter::Io.matches(PageKind::Io, 0));
        assert!(Filter::Program.matches(PageKind::Ram, 1) && !Filter::Program.matches(PageKind::Ram, 0));
        assert_eq!(Filter::parse("rom"), Filter::Rom);
        assert_eq!(View::parse("bytes"), View::Bytes);
    }

    fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }

    #[test]
    fn legende_und_plan_aus_den_zieldaten() {
        let t = vc_target::load_target(&data(), "c64").unwrap();
        let b = t.banking.as_ref().unwrap();
        let plan = b.page_plan(b.default_preset().unwrap());
        assert_eq!(b.default_preset().unwrap().value, 0x37);
        assert_eq!(plan[0xA0].kind, PageKind::BasicRom);
        assert_eq!(plan[0xD0].kind, PageKind::Io);
        assert_eq!(legend(&plan), vec![PageCategory::Zeropage, PageCategory::Stack, PageCategory::Basic, PageCategory::Rom, PageCategory::Io, PageCategory::Free]);
        // Alles RAM: weder BASIC noch E/A in der Legende
        let ram = b.page_plan(b.preset("30").unwrap());
        assert_eq!(legend(&ram), vec![PageCategory::Zeropage, PageCategory::Stack, PageCategory::Free]);
        // Umschalten eines Bits findet das Preset zum Wert
        assert_eq!(b.preset_for_value(0x37 ^ 1).unwrap().id, "36");
        assert!(preset_label(b.preset("37").unwrap()).starts_with("$37  Standard"));
        // Ziele ohne Presets
        assert!(vc_target::load_target(&data(), "vic20").unwrap().banking.is_none());
        let atari = vc_target::load_target(&data(), "atari8").unwrap();
        assert!(preset_label(atari.banking.as_ref().unwrap().preset("e3").unwrap()).ends_with("nur 130XE"));
    }
}
