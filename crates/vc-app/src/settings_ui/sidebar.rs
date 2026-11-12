// SPDX-License-Identifier: GPL-3.0-or-later
//! Linke Leiste: Zielsysteme nach Hersteller gruppiert, Filter und „Zielsystem hinzufügen“.
use super::state::UiState;
use super::widgets::system_color;
use eframe::egui::{self, Color32, RichText};
use vc_ide::settings::Settings;
use vc_ide::targets::{SystemInfo, SYSTEMS};

fn matches(s: &Settings, info: &SystemInfo, q: &str) -> bool {
    let q = q.trim().to_lowercase();
    if q.is_empty() { return true; }
    let mut hay = vec![info.name.to_lowercase(), info.id.to_string(), info.group.to_lowercase()];
    if let Some(t) = s.target(info.id) { for e in &t.emulators { hay.push(e.display_name().to_lowercase()); hay.push(e.kind.clone()); } }
    for m in s.machines_for(info.id) { hay.push(m.display_name().to_lowercase()); hay.push(m.host.to_lowercase()); hay.push(m.device.to_lowercase()); }
    hay.iter().any(|h| h.contains(&q))
}

/// Eintrag der Liste: Symbol, Name, Kurzinfo, Statuspunkt
fn entry(ui: &mut egui::Ui, info: &SystemInfo, selected: bool, sub: &str, dot: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
    let p = ui.painter();
    if selected { p.rect_filled(rect, 8.0, crate::theme::accent().gamma_multiply(0.20)); p.rect_stroke(rect, 8.0, egui::Stroke::new(1.0, crate::theme::accent().gamma_multiply(0.5)), egui::StrokeKind::Inside); }
    else if resp.hovered() { p.rect_filled(rect, 8.0, ui.visuals().widgets.hovered.weak_bg_fill); }
    let b = egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 9.0), egui::vec2(38.0, 26.0));
    p.rect_filled(b, 5.0, system_color(info.id));
    p.text(b.center(), egui::Align2::CENTER_CENTER, info.badge, egui::FontId::monospace(12.0), Color32::WHITE);
    p.text(rect.min + egui::vec2(56.0, 9.0), egui::Align2::LEFT_CENTER, info.name, egui::FontId::proportional(13.5), ui.visuals().strong_text_color());
    p.text(rect.min + egui::vec2(56.0, 28.0), egui::Align2::LEFT_CENTER, sub, egui::FontId::proportional(11.0), ui.visuals().weak_text_color());
    p.circle_filled(rect.right_center() - egui::vec2(12.0, 0.0), 4.0, dot);
    resp
}

pub fn show(ui: &mut egui::Ui, s: &mut Settings, st: &mut UiState) {
    egui::Panel::bottom("set_nav_add").show_inside(ui, |ui| {
        ui.add_space(6.0);
        let free: Vec<&SystemInfo> = SYSTEMS.iter().filter(|i| s.target(i.id).is_none()).collect();
        let mut add = None;
        if free.is_empty() {
            // Nichts mehr hinzuzufügen: lesbar bleiben (kein ausgegrauter Text) und den Grund nennen
            let k = crate::ui::tokens::t();
            ui.add(egui::Button::new(RichText::new("+  Zielsystem hinzufügen").color(k.mute)).sense(egui::Sense::hover())).on_hover_text("Alle Systeme sind bereits eingetragen");
        } else {
            ui.menu_button("+  Zielsystem hinzufügen", |ui| { for i in &free { let hid = s.hidden_targets.iter().any(|h| h.system == i.id); if ui.button(if hid { format!("{} (deaktiviert)", i.name) } else { i.name.to_string() }).clicked() { add = Some(i.id); ui.close(); } } });
        }
        if let Some(id) = add { vc_ide::setup::set_system(s, id, true); st.system = id.into(); }
        ui.add_space(6.0);
    });
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.add_space(6.0);
        ui.add(egui::TextEdit::singleline(&mut st.filter).hint_text("Filter (z. B. VICE, C64, IP) …").desired_width(f32::INFINITY));
        ui.add_space(6.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let mut last_group = "";
            let mut shown = 0;
            for info in SYSTEMS {
                let Some(t) = s.target(info.id) else { continue };
                if !matches(s, info, &st.filter) { continue; }
                shown += 1;
                if info.group != last_group {
                    last_group = info.group;
                    let n = SYSTEMS.iter().filter(|i| i.group == info.group && s.target(i.id).is_some()).count();
                    ui.add_space(6.0);
                    ui.horizontal(|ui| { ui.label(RichText::new(info.group.to_uppercase()).small().strong().weak()); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new(n.to_string()).small().weak()); }); });
                }
                let emu = t.standard_emulator();
                let hw = s.machines_for(info.id).count();
                let mut sub = emu.map_or("kein Emulator".to_string(), |e| e.display_name().to_string());
                if hw > 0 { sub.push_str(&format!(" + {hw} HW")); }
                let dot = match emu { Some(e) if st.presence(e).found => crate::theme::ok(), Some(_) => crate::theme::bad(), None => crate::theme::dim() };
                if entry(ui, info, st.system == info.id, &sub, dot).clicked() { st.system = info.id.into(); }
            }
            if shown == 0 { ui.add_space(8.0); ui.label(RichText::new(if s.targets.is_empty() { "Noch kein Zielsystem. Unten eines hinzufügen." } else { "Nichts gefunden." }).weak()); }
        });
    });
}
