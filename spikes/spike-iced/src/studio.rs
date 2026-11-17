// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: Studiofläche auf `canvas` – Sprite 24×21 mit Raster, Zoom, Zeichnen mit der Maus, Palette.
use crate::theme::Pal;
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke};
use iced::widget::{button, column, container, row, slider, text};
use iced::{mouse, Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme};

pub const W: usize = 24;
pub const H: usize = 21;
pub const COLORS: [(u8, u8, u8); 4] = [(0x20, 0x24, 0x2c), (0xf0, 0xa6, 0x3a), (0x4f, 0x8c, 0xff), (0xff, 0x6b, 0x6b)];

#[derive(Debug, Clone)]
pub enum StudioMsg { Paint(usize, usize, bool), Pick(u8), Zoom(f32), Clear }

pub struct Studio { pub pix: Vec<u8>, pub color: u8, pub zoom: f32, pub cache: Cache }
impl Studio {
    pub fn new() -> Self {
        let mut pix = vec![0u8; W * H];
        for y in 0..H { for x in 0..W { let (dx, dy) = (x as i32 - 12, y as i32 - 10); if dx * dx + dy * dy < 70 { pix[y * W + x] = 1 + (x / 8 % 3) as u8 } } }
        Studio { pix, color: 2, zoom: 22.0, cache: Cache::default() }
    }
    pub fn update(&mut self, m: StudioMsg) -> Task<StudioMsg> {
        match m {
            StudioMsg::Paint(x, y, erase) => { if x < W && y < H { self.pix[y * W + x] = if erase { 0 } else { self.color }; self.cache.clear() } }
            StudioMsg::Pick(c) => self.color = c, StudioMsg::Zoom(z) => { self.zoom = z; self.cache.clear() }
            StudioMsg::Clear => { self.pix.fill(0); self.cache.clear() }
        }
        Task::none()
    }
    pub fn view(&self, pal: Pal) -> Element<'_, StudioMsg> {
        let cv = iced::widget::canvas(Grid { s: self, pal }).width(Length::Fill).height(Length::Fill);
        let side = column![
            text("FARBEN").size(10).color(pal.mute),
            row((0..4u8).map(|i| { let (r, g, b) = COLORS[i as usize]; let sel = self.color == i;
                button(text("")).width(40).height(32).on_press(StudioMsg::Pick(i)).style(move |_, _| button::Style { background: Some(Color::from_rgb8(r, g, b).into()), border: iced::Border { color: if sel { pal.accent } else { pal.line }, width: if sel { 3.0 } else { 1.0 }, radius: 6.0.into() }, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6),
            text(format!("ZOOM {:.0} px", self.zoom)).size(10).color(pal.mute),
            slider(10.0..=34.0, self.zoom, StudioMsg::Zoom).width(200),
            button(text("Leeren").size(13)).on_press(StudioMsg::Clear),
            text("Linke Taste malt, rechte radiert.\nPixel doppelt breit (Mehrfarbe) ist hier nicht abgebildet.").size(11).color(pal.mute),
        ].spacing(10).padding(14).width(260);
        row![container(cv).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), ..Default::default() }), container(side).style(move |_| container::Style { background: Some(pal.panel.into()), ..Default::default() })].into()
    }
    pub fn touch(&self) {}
}

struct Grid<'a> { s: &'a Studio, pal: Pal }
#[derive(Default)]
struct GState { down: Option<bool> }
impl<'a> Grid<'a> {
    fn origin(&self, b: Size) -> Point { Point::new(((b.width - W as f32 * self.s.zoom) / 2.0).max(10.0), ((b.height - H as f32 * self.s.zoom) / 2.0).max(10.0)) }
    fn cell(&self, b: Rectangle, p: Point) -> Option<(usize, usize)> { let o = self.origin(b.size()); let (x, y) = ((p.x - o.x) / self.s.zoom, (p.y - o.y) / self.s.zoom); (x >= 0.0 && y >= 0.0 && x < W as f32 && y < H as f32).then(|| (x as usize, y as usize)) }
}
impl<'a> canvas::Program<StudioMsg> for Grid<'a> {
    type State = GState;
    fn update(&self, st: &mut GState, ev: &canvas::Event, b: Rectangle, cur: mouse::Cursor) -> Option<canvas::Action<StudioMsg>> {
        use canvas::Event::Mouse;
        match ev {
            Mouse(mouse::Event::ButtonPressed(btn @ (mouse::Button::Left | mouse::Button::Right))) => { let erase = *btn == mouse::Button::Right; let p = cur.position_in(b)?; st.down = Some(erase); let (x, y) = self.cell(Rectangle { x: 0.0, y: 0.0, ..b }, p)?; Some(canvas::Action::publish(StudioMsg::Paint(x, y, erase)).and_capture()) }
            Mouse(mouse::Event::CursorMoved { .. }) => { let e = st.down?; let p = cur.position_in(b)?; let (x, y) = self.cell(Rectangle { x: 0.0, y: 0.0, ..b }, p)?; Some(canvas::Action::publish(StudioMsg::Paint(x, y, e))) }
            Mouse(mouse::Event::ButtonReleased(_)) => { st.down = None; None }
            _ => None,
        }
    }
    fn draw(&self, _st: &GState, r: &Renderer, _t: &Theme, b: Rectangle, _c: mouse::Cursor) -> Vec<Geometry> {
        let g = self.s.cache.draw(r, b.size(), |f: &mut Frame| {
            let o = self.origin(b.size()); let z = self.s.zoom;
            for y in 0..H { for x in 0..W { let (cr, cg, cb) = COLORS[self.s.pix[y * W + x] as usize]; let c = if self.s.pix[y * W + x] == 0 && !self.pal.dark { Color::from_rgb8(0xe6, 0xe9, 0xef) } else { Color::from_rgb8(cr, cg, cb) }; f.fill_rectangle(Point::new(o.x + x as f32 * z, o.y + y as f32 * z), Size::new(z - 1.0, z - 1.0), c) } }
            // Raster alle 8 Pixel (Sprite-Byte-Grenzen)
            for i in (0..=W).step_by(8) { f.stroke(&Path::line(Point::new(o.x + i as f32 * z - 0.5, o.y), Point::new(o.x + i as f32 * z - 0.5, o.y + H as f32 * z)), Stroke::default().with_color(self.pal.accent).with_width(1.0)) }
        });
        vec![g]
    }
    fn mouse_interaction(&self, _s: &GState, b: Rectangle, c: mouse::Cursor) -> mouse::Interaction { if c.is_over(b) { mouse::Interaction::Crosshair } else { mouse::Interaction::default() } }
}
