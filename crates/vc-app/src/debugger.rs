// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeichnet die Debugger-Panels im Designsystem (Phase 5, Mockup 6): Statuschip, Registerleiste, Flag-Pillen, Fähigkeiten des Emulators,
//! Disassembler-Zeilen, Speicherzeilen mit Änderungsmarkierung und das Chip-Register-Panel. Zustand und Befehle bleiben in `debug.rs`/`app.rs`.
use crate::debug::{Cmd, DisLine, Snap};
use crate::ui::tokens::{t, tint};
use crate::ui::widgets;
use eframe::egui::{self, text::LayoutJob, Color32, RichText, TextFormat};
use serde_json::Value;
use std::path::Path;

// ---------------------------------------------------------------------------------------------------------------------
// Status und Fähigkeiten

/// Chip „angehalten $0810 datei:zeile“ bzw. „läuft“; `src` = Quellzeile des Programmzählers
pub fn status_chip(ui: &mut egui::Ui, sn: &Snap, src: Option<(String, usize)>) {
    let k = t();
    ui.horizontal_wrapped(|ui| {
        if sn.starting { widgets::chip(ui, "startet …", k.am); }
        else if sn.running { widgets::chip(ui, "läuft", k.gr); }
        else if !sn.have_regs { widgets::chip(ui, "angehalten", k.am); }
        else {
            let mut text = format!("angehalten ${:04X}", sn.pc);
            if let Some((f, n)) = src { text.push_str(&format!(" {f}:{n}")); }
            widgets::chip(ui, &text, k.am);
        }
        if !sn.backend.is_empty() { ui.label(RichText::new(&sn.backend).small().color(k.dim)); }
    });
}

/// Was der Adapter nicht kann, mit Begründung: (Funktion, verfügbar, Begründung)
pub fn limits(sn: &Snap) -> Vec<(&'static str, bool, String)> {
    let b = if sn.backend.is_empty() { "Der Emulator".to_string() } else { sn.backend.clone() };
    let c = sn.caps;
    let atari = b.to_lowercase().contains("atari800");
    vec![
        ("Register", c.registers, format!("{b} liefert keine Registerwerte")),
        ("Einzelschritt", c.step, format!("{b} kennt keinen Einzelschritt")),
        ("Haltepunkte", c.breakpoints, format!("{b} unterstützt keine Haltepunkte")),
        ("Datenhaltepunkte", c.watch, format!("{b} unterstützt keine Datenhaltepunkte (Lesen/Schreiben)")),
        ("Pausieren", sn.can_pause, format!("{b} lässt sich nicht per Knopf anhalten; Haltepunkt setzen oder im Emulatorfenster anhalten")),
        ("mehrere Haltepunkte", !atari && c.breakpoints, if atari { format!("{b} kennt nur einen Haltepunkt zugleich") } else { format!("{b} unterstützt keine Haltepunkte") }),
    ]
}

/// Fähigkeits-Chips: verfügbar in Grün, fehlend gedimmt mit Begründung im Hinweis
pub fn caps_row(ui: &mut egui::Ui, sn: &Snap) {
    let k = t();
    if sn.starting { return; }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (name, ok, why) in limits(sn) {
            let r = ui.scope(|ui| widgets::chip(ui, if ok { name.to_string() } else { format!("{name} ✕") }.as_str(), if ok { k.gr } else { k.dim })).response;
            if !ok { r.on_hover_text(why); }
        }
    });
}

/// Gedimmter Hinweis bei fehlender Funktion
pub fn unavailable(ui: &mut egui::Ui, reason: &str) { ui.label(RichText::new(reason).small().color(t().dim)); }

// ---------------------------------------------------------------------------------------------------------------------
// Register und Flags

/// Registerleiste: PC, A, X, Y, SP als große Kacheln, dahinter die weiteren Register des Adapters (nur, was er wirklich liefert).
/// Liefert (Name, Wert, Antwort) je Kachel für Kontextmenüs.
pub fn reg_tiles(ui: &mut egui::Ui, sn: &Snap) -> Vec<(&'static str, u32, egui::Response)> {
    let mut out = vec![];
    let on = sn.caps.registers && sn.have_regs;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
        for (n, v, w) in [("PC", sn.pc as u32, 4usize), ("A", sn.a as u32, 2), ("X", sn.x as u32, 2), ("Y", sn.y as u32, 2), ("SP", sn.sp as u32, 2)] {
            let txt = if on { format!("{:0w$X}", v, w = w) } else { "–".into() };
            let r = widgets::reg_tile(ui, n, &txt, on);
            out.push((n, v, r));
        }
        for (n, v) in &sn.extra { widgets::reg_tile(ui, n, &format!("{v:X}"), on); }
    });
    out
}

/// Flag-Pillen N V – B D I Z C (nur Anzeige)
pub fn flag_pills(ui: &mut egui::Ui, sn: &Snap) {
    let mut v = sn.flags;
    let on = sn.caps.registers && sn.have_regs;
    ui.add_enabled_ui(on, |ui| {
        let before = v;
        widgets::bitfield(ui, &mut v, &[(7, "N", "Negativ"), (6, "V", "Überlauf"), (5, "-", "unbenutzt"), (4, "B", "Break"), (3, "D", "Dezimalmodus"), (2, "I", "Interrupt gesperrt"), (1, "Z", "Null"), (0, "C", "Übertrag")]);
        let _ = before;   // Klicks ändern nur die lokale Kopie: die Flags sind hier reine Anzeige
    });
}

// ---------------------------------------------------------------------------------------------------------------------
// Disassembler und Speicher

/// Eine Disassembler-Zeile: Cyan-Streifen an der aktuellen Zeile, roter Punkt für Haltepunkte, Zyklen rechts
pub fn disasm_row(ui: &mut egui::Ui, l: &DisLine, label: &str, cur: bool, bp: bool) -> egui::Response {
    let k = t();
    let font = egui::FontId::monospace(13.0);
    let h = ui.text_style_height(&egui::TextStyle::Monospace) + 5.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width().max(380.0), h), egui::Sense::click());
    let p = ui.painter_at(rect);
    if cur { p.rect_filled(rect, 2.0, tint(k.cy)); p.rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())), 1.0, k.cy); }
    else if resp.hovered() { p.rect_filled(rect, 2.0, k.bg3); }
    if bp { p.circle_filled(egui::pos2(rect.left() + 12.0, rect.center().y), 4.5, k.rd); }
    let cw = p.layout_no_wrap("0".into(), font.clone(), k.fg).size().x;
    let y = rect.center().y;
    let mut x = rect.left() + 24.0;
    let put = |text: &str, col: Color32, x: &mut f32, adv: f32| { p.text(egui::pos2(*x, y), egui::Align2::LEFT_CENTER, text, font.clone(), col); *x += adv * cw; };
    put(&format!("{:04X}", l.addr), if cur { k.cy } else { k.dim }, &mut x, 6.0);
    put(&l.bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "), k.mute, &mut x, 10.0);
    if !label.is_empty() { put(&format!("{label}:"), k.am, &mut x, label.chars().count() as f32 + 2.0); }
    put(&l.text, if l.illegal { k.pu } else { k.fg }, &mut x, l.text.chars().count() as f32 + 1.0);
    if l.illegal { put("undok.", k.pu, &mut x, 7.0); }
    if l.cycles > 0 {
        let c = if l.extra { format!("{}+", l.cycles) } else { l.cycles.to_string() };
        p.text(egui::pos2(rect.right() - 8.0, y), egui::Align2::RIGHT_CENTER, c, egui::FontId::monospace(11.5), k.dim);
    }
    resp
}

/// Kopf der Disassembler-Liste
pub fn disasm_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| { ui.add_space(24.0); ui.label(RichText::new("Adresse   Bytes        Befehl").small().color(t().dim)); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new("Zyklen").small().color(t().dim)); }); });
}

/// Speicherzeile: geänderte Bytes gegenüber dem vorherigen Halt in Bernstein
pub fn mem_job(addr: u16, chunk: &[u8], prev: &[u8], row: usize) -> LayoutJob {
    let k = t();
    let mut job = LayoutJob::default();
    let font = egui::FontId::monospace(13.0);
    let f = |c: Color32| TextFormat { font_id: font.clone(), color: c, ..Default::default() };
    job.append(&format!("{addr:04X}  "), 0.0, f(k.dim));
    for (i, b) in chunk.iter().enumerate() {
        let idx = row * 16 + i;
        let changed = prev.get(idx).map_or(false, |p| p != b);
        job.append(&format!("{b:02X} "), 0.0, if changed { TextFormat { background: tint(k.am), ..f(k.am) } } else { f(if *b == 0 { k.dim } else { k.fg }) });
        if i == 7 { job.append(" ", 0.0, f(k.dim)); }
    }
    job.append(" ", 0.0, f(k.dim));
    job.append(&chunk.iter().map(|b| if (0x20..0x7f).contains(b) { *b as char } else { '.' }).collect::<String>(), 0.0, f(k.mute));
    job
}

// ---------------------------------------------------------------------------------------------------------------------
// Chip-Register

pub struct Bit { pub name: String, pub bit: u8, pub width: u8, pub doc: String }
pub struct ChipReg { pub name: String, pub addr: u16, pub access: String, pub doc: String, pub bits: Vec<Bit> }
pub struct ChipDef { pub name: String, pub regs: Vec<ChipReg> }

fn parse_addr(s: &str) -> Option<u32> { let s = s.trim(); if let Some(h) = s.strip_prefix('$') { u32::from_str_radix(h, 16).ok() } else { s.parse().ok() } }

/// Chip-Beschreibung aus `data/chips/<id>.chip.json` (mit Bitfeldern)
pub fn load_chip(data: &Path, id: &str) -> Option<ChipDef> {
    let text = std::fs::read_to_string(data.join("chips").join(format!("{id}.chip.json"))).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let regs = v["regs"].as_array()?.iter().filter_map(|r| {
        let addr = parse_addr(r["addr"].as_str()?)?;
        if addr > 0xFFFF { return None; }
        let bits = r["bits"].as_array().map(|a| a.iter().filter_map(|b| Some(Bit { name: b["name"].as_str()?.into(), bit: b["bit"].as_u64()? as u8, width: b["width"].as_u64().unwrap_or(1) as u8, doc: b["doc"].as_str().unwrap_or("").into() })).collect()).unwrap_or_default();
        Some(ChipReg { name: r["name"].as_str()?.into(), addr: addr as u16, access: r["access"].as_str().unwrap_or("rw").into(), doc: r["doc"].as_str().unwrap_or("").into(), bits })
    }).collect();
    Some(ChipDef { name: v["name"].as_str().unwrap_or(id).into(), regs })
}

/// Chip-Register-Panel. `ids`: Chips des Ziels; die Auswahl liegt im egui-Speicher. Gelesen wird nur bei angehaltenem Emulator
/// (Lesen des CPU-Adressraums); Schreiben ist bewusst nicht möglich, weil Leseseiteneffekte und Schreibregister das Bild verfälschen würden.
pub fn chip_panel(ui: &mut egui::Ui, sn: &Snap, d: &crate::debug::Debugger, data: Option<&Path>, ids: &[String]) {
    let k = t();
    let Some(data) = data else { return };
    if ids.is_empty() { return; }
    let key = egui::Id::new("chip_sel");
    let mut sel: String = ui.ctx().data(|m| m.get_temp(key)).unwrap_or_else(|| ids[0].clone());
    if !ids.contains(&sel) { sel = ids[0].clone(); }
    egui::CollapsingHeader::new(RichText::new("Chip-Register").strong()).id_salt("chip_regs").default_open(true).show(ui, |ui| {
        egui::ComboBox::from_id_salt("chip_combo").selected_text(&sel).show_ui(ui, |ui| { for i in ids { ui.selectable_value(&mut sel, i.clone(), i.as_str()); } });
        let Some(def) = load_chip(data, &sel) else { unavailable(ui, "Chip-Beschreibung nicht lesbar"); return };
        ui.label(RichText::new(&def.name).small().color(k.mute));
        let (lo, hi) = (def.regs.iter().map(|r| r.addr).min().unwrap_or(0), def.regs.iter().map(|r| r.addr).max().unwrap_or(0));
        let len = hi as u32 - lo as u32 + 1;
        if len > 0x1000 { unavailable(ui, "Der Adressbereich dieses Chips ist zu groß für die Anzeige"); return; }
        // Lesefenster beim Emulator anmelden, wenn es sich ändert
        let want = (lo, len as u16);
        let sent: Option<(u16, u16)> = ui.ctx().data(|m| m.get_temp(egui::Id::new("chip_sent")));
        if sent != Some(want) { d.send(Cmd::Chips(vec![want])); ui.ctx().data_mut(|m| m.insert_temp(egui::Id::new("chip_sent"), want)); }
        let can_read = sn.caps.registers && !sn.running;
        if !sn.caps.registers { unavailable(ui, &format!("{} kann den Speicher nicht im angehaltenen Zustand lesen – keine Werte.", if sn.backend.is_empty() { "Der Emulator" } else { &sn.backend })); }
        else if sn.running { unavailable(ui, "Der Emulator läuft: Werte erscheinen beim nächsten Halt."); }
        let data_bytes = sn.chips.iter().find(|(a, b)| *a == lo && b.len() == len as usize).map(|(_, b)| b.as_slice());
        egui::ScrollArea::vertical().max_height(320.0).id_salt("chip_scroll").auto_shrink([false, true]).show(ui, |ui| {
            for r in &def.regs {
                let readable = r.access != "w";
                let val = data_bytes.filter(|_| can_read && readable).and_then(|b| b.get((r.addr - lo) as usize)).copied();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("${:04X}", r.addr)).monospace().color(k.dim));
                    ui.label(RichText::new(&r.name).monospace().color(k.fg)).on_hover_text(&r.doc);
                    match val {
                        Some(v) => { ui.label(RichText::new(format!("{v:02X}")).monospace().strong().color(k.gr)); }
                        None => { ui.label(RichText::new("–").monospace().color(k.dim)).on_hover_text(if !readable { "Nur Schreibregister: der Wert ist nicht lesbar" } else { "Kein Wert (Emulator läuft oder liefert keine Daten)" }); }
                    }
                    if let Some(v) = val {
                        // einzelne Bits als Schalter (nur Anzeige), Bitgruppen als Wert
                        let singles: Vec<(u8, &str, &str)> = r.bits.iter().filter(|b| b.width <= 1).map(|b| (b.bit, b.name.as_str(), b.doc.as_str())).collect();
                        if !singles.is_empty() { let mut c = v; widgets::bitfield(ui, &mut c, &singles); }
                        for b in r.bits.iter().filter(|b| b.width > 1) {
                            let mask = ((1u16 << b.width) - 1) as u8;
                            widgets::mono_chip(ui, &format!("{}={}", b.name, (v >> b.bit) & mask), k.cy);
                        }
                    }
                });
            }
        });
        ui.label(RichText::new("Nur Anzeige. Gelesen wird die Sicht der CPU beim Halt; Schreibregister zeigen „–“.").small().color(k.dim));
    });
    ui.ctx().data_mut(|m| m.insert_temp(key, sel));
}

/// Beispielwerte für den Selbsttest ohne laufenden Emulator (`VC_DEMO=dbgdemo`, Emulator über `VC_DBG_BACKEND=vice|atari800|ultimate`): ausdrücklich keine Messung
pub fn demo_snap(backend: &str) -> Snap {
    use crate::debug::{CapsInfo, DisLine, PointInfo};
    let dis = |addr: u16, bytes: &[u8], text: &str, cycles: u8, extra: bool| DisLine { addr, bytes: bytes.to_vec(), text: text.into(), illegal: false, cycles, extra };
    Snap {
        have_regs: backend != "ultimate", backend: match backend { "ultimate" => "Ultimate 64 (Beispiel)", "atari800" => "atari800 (Beispiel)", _ => "VICE (Beispiel)" }.into(), pc: 0x0810, a: 0x1B, x: 0x00, y: 0xFF, sp: 0xF8, flags: 0b1010_0100,
        caps: match backend { "ultimate" => CapsInfo::default(), "atari800" => CapsInfo { registers: true, step: true, breakpoints: true, watch: false }, _ => CapsInfo { registers: true, step: true, breakpoints: true, watch: true } },
        can_pause: backend == "vice",
        extra: if backend == "vice" { vec![("RL".into(), 0x0A8), ("CYC".into(), 0x14)] } else { vec![] },
        disasm: vec![dis(0x0810, &[0xA9, 0x1B], "lda #$1b", 2, false), dis(0x0812, &[0x8D, 0x20, 0xD0], "sta $d020", 4, false), dis(0x0815, &[0xD0, 0xF9], "bne $0810", 2, true), dis(0x0817, &[0x4C, 0x17, 0x08], "jmp $0817", 3, false)],
        breakpoints: vec![0x0812], points: vec![PointInfo { id: 1, kind: "exec".into(), start: 0x0812, end: 0x0812, ..Default::default() }],
        mem_addr: 0x0800, mem_prev: (0..32u8).collect(), mem: (0..32u8).map(|i| if i == 5 || i == 18 { i ^ 0x5A } else { i }).collect(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_mit_bitfeldern_wird_gelesen() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let c = load_chip(&data, "vic2").unwrap();
        let ctrl1 = c.regs.iter().find(|r| r.name == "ctrl1").unwrap();
        assert_eq!(ctrl1.addr, 0xD011);
        assert!(ctrl1.bits.iter().any(|b| b.name == "YSCROLL" && b.width == 3));
        assert!(load_chip(&data, "gibt-es-nicht").is_none());
    }

    #[test]
    fn grenzen_je_emulator() {
        let off = |sn: &Snap, n: &str| limits(sn).into_iter().find(|l| l.0 == n).map(|l| !l.1).unwrap();
        let (vice, atari, ult) = (demo_snap("vice"), demo_snap("atari800"), demo_snap("ultimate"));
        assert!(!off(&vice, "Datenhaltepunkte") && !off(&vice, "mehrere Haltepunkte") && !off(&vice, "Pausieren"));
        assert!(off(&atari, "Datenhaltepunkte") && off(&atari, "mehrere Haltepunkte") && off(&atari, "Pausieren") && !off(&atari, "Register"));
        assert!(["Register", "Einzelschritt", "Haltepunkte"].iter().all(|n| off(&ult, n)));
        assert!(limits(&ult).iter().filter(|l| !l.1).all(|l| !l.2.is_empty()));
    }

    #[test]
    fn panels_zeichnen_mit_beispieldaten() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        for backend in ["vice", "atari800", "ultimate"] {
            let sn = demo_snap(backend);
            let d = crate::debug::Debugger::demo(sn.clone());
            for _ in 0..2 {
                let mut o = ctx.run_ui(egui::RawInput::default(), |ui| {
                    status_chip(ui, &sn, Some(("main.asm".into(), 12)));
                    caps_row(ui, &sn);
                    assert_eq!(reg_tiles(ui, &sn).len(), 5);
                    flag_pills(ui, &sn);
                    disasm_header(ui);
                    for (i, l) in sn.disasm.iter().enumerate() { disasm_row(ui, l, "start", i == 0, sn.breakpoints.contains(&l.addr)); }
                    for (row, chunk) in sn.mem.chunks(16).enumerate() { ui.label(mem_job(0x0800 + row as u16 * 16, chunk, &sn.mem_prev, row)); }
                    chip_panel(ui, &sn, &d, Some(&data), &["vic2".to_string(), "sid".to_string()]);
                });
                o.textures_delta.clear();
            }
        }
    }
}
