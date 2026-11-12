// SPDX-License-Identifier: GPL-3.0-or-later
//! Eigene Titelzeile im Designsystem. Unter macOS reicht der Inhalt bis zum oberen Fensterrand, die Ampelknöpfe des Systems
//! bleiben und liegen über dem linken Freiraum. Die freie Fläche der Zeile zieht das Fenster, ein Doppelklick zoomt es.
//! Unter Windows und Linux bleibt die Titelleiste des Systems; dort ist `active()` falsch und nichts davon wirkt.
use super::tokens::t;
use super::widgets::chip;
use eframe::egui::{self, Color32, Rect, RichText};

/// Höhe der Titelzeile
pub const HEIGHT: f32 = 32.0;
/// Freiraum links für die Ampelknöpfe des Systems
pub const LEFT_INSET: f32 = 76.0;

/// Ob die Oberfläche die Titelzeile selbst zeichnet
pub const fn active() -> bool { cfg!(target_os = "macos") }

/// Fensterattribute: Inhalt bis zum Rand, Systemtitel ausgeblendet, Ampelknöpfe bleiben. Der Fenstertitel selbst bleibt gesetzt
/// (Dock, Fenstermenü, Barrierefreiheit). Außerhalb von macOS unverändert.
pub fn decorate(b: egui::ViewportBuilder) -> egui::ViewportBuilder {
    #[cfg(target_os = "macos")]
    { b.with_fullsize_content_view(true).with_titlebar_shown(false).with_title_shown(false).with_titlebar_buttons_shown(true) }
    #[cfg(not(target_os = "macos"))]
    { b }
}

/// Die Fläche, die das Fenster zieht: die ganze Zeile
pub fn drag_rect(bar: Rect) -> Rect { Rect::from_min_size(bar.min, egui::vec2(bar.width(), HEIGHT.min(bar.height()))) }

/// Ziehfläche anlegen. Vor den Inhalten der Zeile aufrufen: Knöpfe und Felder darüber haben Vorrang, nur freie Fläche zieht.
pub fn drag_area(ui: &mut egui::Ui, bar: Rect, id: egui::Id) {
    let r = ui.interact(drag_rect(bar), id, egui::Sense::click_and_drag());
    if r.drag_started_by(egui::PointerButton::Primary) { ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag); }
    if r.double_clicked() {
        let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
    }
}

/// Titelzeile eines Studio-Fensters: Freiraum für die Ampelknöpfe, Dateiname, Art. Liefert das Rechteck der Zeile.
pub fn bar(ui: &mut egui::Ui, id: &str, inset: f32, file: &str, kind: &str, kind_color: Color32) -> Rect {
    let k = t();
    let r = egui::Panel::top(egui::Id::new(("titlebar", id))).exact_size(HEIGHT)
        .frame(egui::Frame::new().fill(k.bg1).inner_margin(egui::Margin::symmetric(10, 0)))
        .show_inside(ui, |ui| {
            let area = ui.max_rect();
            drag_area(ui, area, egui::Id::new(("titlebar-drag", id)));
            ui.horizontal_centered(|ui| {
                ui.add_space(inset);
                if !file.is_empty() { ui.label(RichText::new(file).font(egui::FontId::new(13.0, crate::fonts::mono_strong())).color(k.fg)); }
                if !kind.is_empty() { chip(ui, kind, kind_color); }
            });
        });
    r.response.rect
}

/// Systemthema des Fensters an das Thema der Oberfläche angleichen (färbt die Ampelknöpfe und den Rahmen); nur bei Änderung senden
pub fn sync_theme(ctx: &egui::Context) {
    if !active() { return; }
    let want = match ctx.options(|o| o.theme_preference) {
        egui::ThemePreference::Dark => egui::viewport::SystemTheme::Dark,
        egui::ThemePreference::Light => egui::viewport::SystemTheme::Light,
        egui::ThemePreference::System => egui::viewport::SystemTheme::SystemDefault,
    };
    let key = egui::Id::new(("titlebar-theme", ctx.viewport_id()));
    let last: Option<egui::viewport::SystemTheme> = ctx.data(|d| d.get_temp(key));
    if last != Some(want) { ctx.data_mut(|d| d.insert_temp(key, want)); ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(want)); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn frame(ctx: &egui::Context, size: egui::Vec2, events: Vec<egui::Event>, body: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
        ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), size)), events, ..Default::default() }, body)
    }
    fn commands(o: &egui::FullOutput) -> Vec<egui::ViewportCommand> { o.viewport_output.values().flat_map(|v| v.commands.clone()).collect() }
    fn click(p: egui::Pos2, pressed: bool) -> egui::Event { egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() } }

    #[test]
    fn zeile_hat_hoehe_und_freiraum() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut rect = Rect::NOTHING;
        for _ in 0..3 {
            let mut o = frame(&ctx, vec2(600.0, 400.0), vec![], |ui| {
                rect = bar(ui, "t", LEFT_INSET, "level.vcscreen", "Bildschirme", t().cy);
            });
            o.textures_delta.clear();
        }
        assert_eq!(rect.height(), HEIGHT);
        assert_eq!(rect.top(), 0.0);
        assert!(rect.width() >= 600.0 - 1.0);
        assert!(LEFT_INSET >= 70.0 && LEFT_INSET <= 80.0);
    }

    #[test]
    fn freie_flaeche_zieht_das_fenster() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let size = vec2(600.0, 400.0);
        let p = pos2(500.0, 16.0); // rechts, frei
        let run = |ev: Vec<egui::Event>| { let mut o = frame(&ctx, size, ev, |ui| { bar(ui, "t", LEFT_INSET, "a.json", "Art", t().cy); }); o.textures_delta.clear(); commands(&o) };
        run(vec![]);
        run(vec![egui::Event::PointerMoved(p)]);
        run(vec![click(p, true)]);
        let c = run(vec![egui::Event::PointerMoved(p + vec2(8.0, 3.0))]);
        assert!(c.iter().any(|c| matches!(c, egui::ViewportCommand::StartDrag)), "Ziehen auf freier Fläche startet das Verschieben: {c:?}");
    }

    #[test]
    fn inhalt_darunter_zieht_nicht() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let size = vec2(600.0, 400.0);
        let p = pos2(300.0, HEIGHT + 40.0); // unterhalb der Zeile
        let run = |ev: Vec<egui::Event>| { let mut o = frame(&ctx, size, ev, |ui| { bar(ui, "t", LEFT_INSET, "a.json", "Art", t().cy); }); o.textures_delta.clear(); commands(&o) };
        run(vec![]);
        run(vec![egui::Event::PointerMoved(p)]);
        run(vec![click(p, true)]);
        let c = run(vec![egui::Event::PointerMoved(p + vec2(8.0, 3.0))]);
        assert!(!c.iter().any(|c| matches!(c, egui::ViewportCommand::StartDrag)));
    }

    #[test]
    fn ziehflaeche_ist_hoechstens_so_hoch_wie_die_zeile() {
        let big = Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 900.0));
        assert_eq!(drag_rect(big).height(), HEIGHT);
        assert_eq!(drag_rect(big).width(), 500.0);
    }
}
