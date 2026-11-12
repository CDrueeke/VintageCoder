// SPDX-License-Identifier: GPL-3.0-or-later
//! Startparameter eines Emulators aus dem Katalog (`data/args`): gruppierte Schalter, Auswahlen und Felder mit Hover-Text, freies Feld
//! „Weitere Parameter“, anklickbare Platzhalter, Vorschau der fertigen Kommandozeile und Prüfmeldungen. Rechnen tut `vc_ide::args`;
//! hier wird nur die Auswahl im Entwurf (`EmulatorCfg::opts`/`args`) geändert.
use super::{state::UiState, widgets};
use crate::ui::tokens::t;
use eframe::egui::{self, RichText};
use std::sync::Arc;
use vc_ide::args::{self, Catalog, Level, Param, ParamType, Selection, PLACEHOLDERS};
use vc_ide::targets::EmulatorCfg;

/// Schlüssel des freien Feldes für die Platzhalter-Einfügung
const FREE: &str = "";

/// Wert eines Parameters in der Auswahl lesen bzw. setzen (leer = entfernen)
fn set(opts: &mut Selection, id: &str, v: &str) { if v.is_empty() { opts.remove(id); } else { opts.insert(id.to_string(), v.to_string()); } }

fn flag_text(p: &Param) -> String { match &p.off_flag { Some(o) => format!("{} / {}", p.flag, o), None => p.flag.clone() } }

fn hover(p: &Param) -> String {
    let mut s = format!("{}\n\nSchalter: {}", p.hint, flag_text(p));
    if let Some(d) = &p.default { s.push_str(&format!("\nStandard des Emulators: {d}")); }
    if let Some(x) = &p.example { s.push_str(&format!("\nBeispiel: {x}")); }
    if p.min.is_some() || p.max.is_some() { s.push_str(&format!("\nBereich: {} bis {}", p.min.map_or("–".into(), |m| m.to_string()), p.max.map_or("–".into(), |m| m.to_string()))); }
    if !p.conflicts.is_empty() { s.push_str("\nNicht zusammen mit: "); s.push_str(&p.conflicts.join(", ")); }
    if !p.requires.is_empty() { s.push_str("\nBraucht: "); s.push_str(&p.requires.join(", ")); }
    s
}

/// Eine Zeile je Parameter; liefert, ob ein Textfeld den Fokus hat (für die Platzhalter-Einfügung)
fn param_row(ui: &mut egui::Ui, owner: &str, opts: &mut Selection, p: &Param, focus: &mut Option<(String, String)>) {
    let cur = opts.get(&p.id).cloned().unwrap_or_default();
    let mut new = cur.clone();
    let k = t();
    ui.horizontal(|ui| {
        // zurücksetzen („×“) nur, wenn gesetzt
        let reset = ui.add_visible(!cur.is_empty(), egui::Button::new(RichText::new("×").color(k.mute)).small().frame(false)).on_hover_text("Zurück auf den Standard des Emulators (nicht setzen)");
        if reset.clicked() { new.clear(); }
        match p.ty {
            ParamType::Switch if p.off_flag.is_none() => {
                let mut on = cur == "on";
                if ui.checkbox(&mut on, &p.label).on_hover_text(hover(p)).changed() { new = if on { "on".into() } else { String::new() }; }
            }
            ParamType::Switch => {
                let txt = match cur.as_str() { "on" => "an", "off" => "aus", _ => "Standard" };
                egui::ComboBox::from_id_salt(("opt", owner, &p.id)).width(90.0).selected_text(txt).show_ui(ui, |ui| {
                    for (v, l) in [("", "Standard"), ("on", "an"), ("off", "aus")] { if ui.selectable_label(cur == v, l).clicked() { new = v.into(); } }
                });
                ui.label(&p.label).on_hover_text(hover(p));
            }
            ParamType::Choice => {
                let txt = p.choices.iter().find(|c| c.value == cur).map(|c| c.label.clone()).unwrap_or_else(|| if cur.is_empty() { "Standard".into() } else { cur.clone() });
                egui::ComboBox::from_id_salt(("opt", owner, &p.id)).width(190.0).selected_text(txt).show_ui(ui, |ui| {
                    if ui.selectable_label(cur.is_empty(), "Standard").clicked() { new.clear(); }
                    for c in &p.choices { if ui.selectable_label(cur == c.value, format!("{}  ({})", c.label, c.value)).clicked() { new = c.value.clone(); } }
                });
                ui.label(&p.label).on_hover_text(hover(p));
            }
            ParamType::Text | ParamType::Number | ParamType::File => {
                let w = if p.ty == ParamType::Number { 90.0 } else { 260.0 };
                let r = ui.add(egui::TextEdit::singleline(&mut new).desired_width(w).font(egui::TextStyle::Monospace).hint_text(p.example.clone().unwrap_or_default()));
                if r.has_focus() || r.gained_focus() { *focus = Some((owner.to_string(), p.id.clone())); }
                r.on_hover_text(hover(p));
                if p.ty == ParamType::File && ui.small_button("…").on_hover_text("Datei wählen").clicked() { if let Some(f) = rfd::FileDialog::new().pick_file() { new = f.to_string_lossy().into_owned(); } }
                ui.label(&p.label).on_hover_text(hover(p));
            }
        }
        ui.label(RichText::new(flag_text(p)).small().monospace().color(k.dim));
    });
    if new != cur { set(opts, &p.id, &new); }
}

fn groups_ui(ui: &mut egui::Ui, owner: &str, opts: &mut Selection, cat: &Catalog, focus: &mut Option<(String, String)>) {
    for g in &cat.groups {
        let n_all = cat.in_group(&g.id).count();
        if n_all == 0 { continue; }
        let set_n = cat.in_group(&g.id).filter(|p| opts.get(&p.id).map_or(false, |v| !v.trim().is_empty())).count();
        let title = if set_n > 0 { format!("{}   ·   {} gesetzt", g.label, set_n) } else { g.label.clone() };
        egui::CollapsingHeader::new(RichText::new(title).strong()).id_salt(("argroup", owner, &g.id)).default_open(set_n > 0).show(ui, |ui| {
            for p in cat.in_group(&g.id) { param_row(ui, owner, opts, p, focus); }
        });
    }
}

/// Parametereditor eines Emulators. `system` bestimmt die Beispielwerte der Platzhalter, `program` die Kopfzeile der Vorschau.
pub fn show(ui: &mut egui::Ui, e: &mut EmulatorCfg, system: &str, program: &str, st: &mut UiState) {
    let cat = args::catalog_of_kind(&e.kind);
    let hint = match e.kind.as_str() { "altirra" => "z. B. /f", "atari800" => "z. B. -nosound", "xemu" => "z. B. -fullscreen", _ => "z. B. -VICIIfilter 0 -sidenginemodel 1" };
    editor(ui, &Spec { owner: &e.id, title: "Startparameter", cat, ctx: args::ArgCtx::example(system), program, hint, reserved_note: true }, &mut e.opts, &mut e.args, &mut st.focus);
}

/// Was ein Parametereditor außer den Werten braucht
pub struct Spec<'a> { pub owner: &'a str, pub title: &'a str, pub cat: Option<Arc<Catalog>>, pub ctx: args::ArgCtx, pub program: &'a str, pub hint: &'a str, pub reserved_note: bool }

/// Gruppierte Parameter, freies Feld, Platzhalter, Prüfung und Vorschau für eine Auswahl `opts` und ein freies Feld `free`
pub fn editor(ui: &mut egui::Ui, sp: &Spec, opts: &mut Selection, free: &mut String, focus: &mut Option<(String, String)>) {
    let k = t();
    let (cat, ctx, owner) = (&sp.cat, &sp.ctx, sp.owner);
    let n_set = cat.as_deref().map_or(0, |c| args::changed_count(c, opts));
    let head = if n_set > 0 { format!("{}   ·   {n_set} gesetzt", sp.title) } else { sp.title.to_string() };
    egui::CollapsingHeader::new(RichText::new(head).font(egui::FontId::new(13.0, crate::fonts::ui_strong()))).id_salt(("args", owner)).default_open(n_set > 0 || !free.trim().is_empty()).show(ui, |ui| {
        match &cat {
            Some(c) => {
                ui.label(RichText::new(format!("Katalog {} – nicht gesetzte Parameter bleiben beim Standard. {}", c.label, if c.verified.starts_with("nicht geprüft") { "Achtung: dieser Katalog ist nicht gegen das Programm geprüft." } else { "" })).small().color(k.mute));
                ui.add_space(2.0);
                groups_ui(ui, owner, opts, c, focus);
            }
            None => { ui.label(RichText::new("Für diese Emulator-Sorte gibt es keinen Katalog (data/args). Parameter lassen sich im freien Feld eintragen.").small().color(k.mute)); }
        }
        ui.add_space(6.0);
        // freies Feld
        ui.horizontal(|ui| {
            widgets::label(ui, "Weitere Parameter");
            let r = ui.add(egui::TextEdit::singleline(free).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace).text_color(crate::theme::warn()).hint_text(sp.hint));
            if r.has_focus() || r.gained_focus() { *focus = Some((owner.to_string(), FREE.to_string())); }
            r.on_hover_text("Alles, was der Katalog nicht kennt, wird wie in der Shell geschrieben angehängt (nach den Katalogparametern). Platzhalter wie {project} sind erlaubt.");
        });
        // Platzhalter
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Variable einfügen:").small().color(k.mute));
            ui.spacing_mut().item_spacing.x = 4.0;
            for p in PLACEHOLDERS {
                let ex = args::expand(&format!("{{{}}}", p.name), &ctx).unwrap_or_else(|_| "(nicht in jedem Projekt vorhanden)".into());
                let b = egui::Button::new(RichText::new(format!("{{{}}}", p.name)).small().monospace().color(crate::theme::accent())).fill(crate::ui::tokens::tint(k.cy)).stroke(egui::Stroke::new(1.0, k.cy.gamma_multiply(0.30))).corner_radius(99.0);
                if ui.add(b).on_hover_text(format!("{}: {}\nBeispiel: {}\nFügt den Platzhalter am Ende des zuletzt bearbeiteten Feldes ein.", p.label, p.hint, ex)).clicked() {
                    let key = match &*focus { Some((id, f)) if id == owner => f.clone(), _ => FREE.to_string() };
                    let ph = format!("{{{}}}", p.name);
                    if key.is_empty() { if !free.is_empty() && !free.ends_with(' ') { free.push(' '); } free.push_str(&ph); }
                    else { opts.entry(key).or_default().push_str(&ph); }
                }
            }
        });
        // Übernahme von Freitext
        if let Some(c) = cat.as_deref() {
            let n = args::migratable(c, opts, free);
            if n > 0 {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{n} Parameter im freien Feld kennt der Katalog.")).small().color(k.mute));
                    if ui.small_button("In die Auswahl übernehmen").on_hover_text("Verschiebt bekannte Parameter aus dem freien Feld in die Auswahl; die Kommandozeile bleibt gleichwertig.").clicked() {
                        let (s, rest) = args::migrate_free(c, opts, free); *opts = s; *free = rest;
                    }
                });
            }
        }
        // Prüfung
        let issues = args::check(cat.as_deref(), opts, free, Some(ctx));
        for i in &issues {
            let (sym, col) = match i.level { Level::Error => ("✖", crate::theme::bad()), Level::Warn => ("⚠", crate::theme::warn()), Level::Info => ("ℹ", k.mute) };
            ui.label(RichText::new(format!("{sym} {}", i.text)).small().color(col));
        }
        // Vorschau
        ui.add_space(4.0);
        let (resolved, _) = args::preview(cat.as_deref(), opts, free, ctx);
        let line = args::shell_line(sp.program, &resolved);
        egui::Frame::new().fill(k.bg0).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Vorschau (Beispielprojekt „demo“)").small().color(k.mute));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.small_button("Kopieren").on_hover_text("Kopiert die Kommandozeile in die Zwischenablage").clicked() { ui.ctx().copy_text(line.clone()); } });
            });
            ui.add(egui::Label::new(RichText::new(&line).monospace().small()).wrap().selectable(true));
            if let Some(c) = cat.as_deref().filter(|_| sp.reserved_note) { if !c.reserved.is_empty() { ui.label(RichText::new(format!("Beim Starten setzt VintageCoder selbst: {}", c.reserved.iter().take(6).cloned().collect::<Vec<_>>().join(" "))).small().color(k.dim)); } }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let changed = !opts.is_empty() || !free.trim().is_empty();
            if ui.add_enabled(changed, egui::Button::new("Auf Standard zurücksetzen")).on_hover_text("Entfernt alle gesetzten Parameter und leert das freie Feld; der Emulator läuft dann mit seinen eigenen Standardwerten").clicked() { args::reset(opts, free); }
        });
    });
}
