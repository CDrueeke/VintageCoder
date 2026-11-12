// SPDX-License-Identifier: GPL-3.0-or-later
//! Kleine Symbole für Befehlsleiste und Aktivitätsleiste, mit dem Zeichenstift gemalt (kein Symbolfont nötig, in Dunkel und Hell gleich scharf).
use eframe::egui::{self, Color32, Pos2, Stroke};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon { Play, Pause, Stop, Cont, Bug, Hammer, Search, Folder, Flask, Book, Gear, StepInto, StepOver, StepOut }

/// Zeichnet `icon` in ein Quadrat der Kantenlänge `s` um `c`
pub fn paint(p: &egui::Painter, c: Pos2, s: f32, icon: Icon, col: Color32) {
    let u = s / 16.0;                                   // Einheit: 1/16 der Kante
    let at = |x: f32, y: f32| Pos2::new(c.x + x * u, c.y + y * u);
    let st = Stroke::new((1.4 * u).max(1.2), col);
    match icon {
        Icon::Play => { p.add(egui::Shape::convex_polygon(vec![at(-4.0, -6.0), at(-4.0, 6.0), at(6.0, 0.0)], col, Stroke::NONE)); }
        Icon::Pause => { p.rect_filled(egui::Rect::from_min_max(at(-5.0, -6.0), at(-1.5, 6.0)), 1.0, col); p.rect_filled(egui::Rect::from_min_max(at(1.5, -6.0), at(5.0, 6.0)), 1.0, col); }
        Icon::Stop => { p.rect_filled(egui::Rect::from_min_max(at(-5.0, -5.0), at(5.0, 5.0)), 1.5, col); }
        Icon::Cont => { p.rect_filled(egui::Rect::from_min_max(at(-6.0, -6.0), at(-3.5, 6.0)), 1.0, col); p.add(egui::Shape::convex_polygon(vec![at(-1.0, -6.0), at(-1.0, 6.0), at(7.0, 0.0)], col, Stroke::NONE)); }
        Icon::Bug => {
            p.circle_stroke(at(0.0, 1.0), 4.2 * u, st);
            p.circle_filled(at(0.0, -4.0), 2.0 * u, col);
            for y in [-1.0, 2.0, 5.0] { p.line_segment([at(-4.2, y), at(-7.0, y - 1.0)], st); p.line_segment([at(4.2, y), at(7.0, y - 1.0)], st); }
        }
        Icon::Hammer => { p.line_segment([at(-6.0, 6.0), at(2.0, -2.0)], Stroke::new((2.2 * u).max(1.6), col)); p.add(egui::Shape::convex_polygon(vec![at(-1.0, -7.0), at(6.0, -1.0), at(3.0, 2.0), at(-4.0, -4.0)], col, Stroke::NONE)); }
        Icon::Search => { p.circle_stroke(at(-1.5, -1.5), 4.5 * u, st); p.line_segment([at(1.8, 1.8), at(6.5, 6.5)], st); }
        Icon::Folder => { p.rect_stroke(egui::Rect::from_min_max(at(-7.0, -4.0), at(7.0, 6.0)), 1.5, st, egui::StrokeKind::Middle); p.line_segment([at(-7.0, -4.0), at(-7.0, -6.0)], st); p.line_segment([at(-7.0, -6.0), at(-2.0, -6.0)], st); p.line_segment([at(-2.0, -6.0), at(-0.5, -4.0)], st); }
        Icon::Flask => { p.line_segment([at(-2.0, -7.0), at(-2.0, -1.0)], st); p.line_segment([at(2.0, -7.0), at(2.0, -1.0)], st); p.line_segment([at(-3.0, -7.0), at(3.0, -7.0)], st); p.line_segment([at(-2.0, -1.0), at(-6.5, 6.0)], st); p.line_segment([at(2.0, -1.0), at(6.5, 6.0)], st); p.line_segment([at(-6.5, 6.0), at(6.5, 6.0)], st); }
        Icon::Book => { p.rect_stroke(egui::Rect::from_min_max(at(-6.0, -6.0), at(6.0, 6.0)), 1.5, st, egui::StrokeKind::Middle); p.line_segment([at(0.0, -6.0), at(0.0, 6.0)], st); p.line_segment([at(-4.0, -2.5), at(-2.0, -2.5)], st); p.line_segment([at(2.0, -2.5), at(4.0, -2.5)], st); }
        Icon::Gear => {
            p.circle_stroke(at(0.0, 0.0), 3.0 * u, st);
            for i in 0..8 { let a = i as f32 * std::f32::consts::FRAC_PI_4; let (sn, cs) = a.sin_cos(); p.line_segment([at(cs * 5.0, sn * 5.0), at(cs * 7.0, sn * 7.0)], Stroke::new((2.0 * u).max(1.4), col)); }
        }
        Icon::StepInto => { p.line_segment([at(0.0, -7.0), at(0.0, 2.0)], st); p.line_segment([at(-4.0, -2.0), at(0.0, 2.0)], st); p.line_segment([at(4.0, -2.0), at(0.0, 2.0)], st); p.circle_filled(at(0.0, 6.0), 1.6 * u, col); }
        Icon::StepOut => { p.line_segment([at(0.0, 2.0), at(0.0, -7.0)], st); p.line_segment([at(-4.0, -3.0), at(0.0, -7.0)], st); p.line_segment([at(4.0, -3.0), at(0.0, -7.0)], st); p.circle_filled(at(0.0, 6.0), 1.6 * u, col); }
        Icon::StepOver => { p.line_segment([at(-6.0, 3.0), at(-6.0, -1.0)], st); p.line_segment([at(-6.0, -1.0), at(-3.0, -4.0)], st); p.line_segment([at(-3.0, -4.0), at(3.0, -4.0)], st); p.line_segment([at(3.0, -4.0), at(6.0, -1.0)], st); p.line_segment([at(6.0, -1.0), at(6.0, 1.0)], st); p.line_segment([at(3.0, 0.0), at(6.0, 3.5)], st); p.line_segment([at(9.0, 0.0), at(6.0, 3.5)], st); p.circle_filled(at(-6.0, 6.0), 1.6 * u, col); }
    }
}
