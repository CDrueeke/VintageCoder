// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: Editor als eigenes Iced-Widget (Trait `iced::advanced::Widget`). Festes Zeichenraster, eigener Rand mit Zeilennummern,
//! Haltepunkten und Faltmarken, Auswahl, Fehlerwellen, Zeilenfarben, Vervollständigungsliste, IME, Mausauswahl.
use crate::core::{col_at_vis, vis_col, vis_len, Editor, Pos, TAB};
use crate::highlight::{tokens, Kind};
use crate::theme::Pal;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::{self, Paragraph as _, Renderer as _, Text};
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{clipboard, input_method, mouse, Clipboard, InputMethod, Shell, Widget};
use iced::keyboard::{self, key::Named, Key};
use iced::{alignment, window, Border, Color, Element, Event, Font, Length, Pixels, Point, Rectangle, Size};
use std::sync::atomic::{AtomicU64, Ordering};

pub static DRAW_NS: AtomicU64 = AtomicU64::new(0);
pub static DRAW_N: AtomicU64 = AtomicU64::new(0);

pub const GUTTER: f32 = 62.0;
const PAD: f32 = 8.0;

#[derive(Debug, Clone)]
pub enum EdMsg {
    Insert(String), Backspace, Delete, Enter, Tab,
    Move { dx: i32, dy: i32, extend: bool }, Home(bool), End(bool), Page(i32, bool),
    SelectAll, Undo, Redo, Click(Pos, bool, bool), Drag(Pos), Breakpoint(usize), Fold(usize), Scroll(f32),
    Resize(usize, usize), CompletionMove(i32), CompletionAccept, CompletionClose, Preedit(Option<String>), ToggleWrap,
}

pub struct EditorView<'a, M> { ed: &'a Editor, pal: Pal, size: f32, on: Box<dyn Fn(EdMsg) -> M + 'a> }
pub fn editor_view<'a, M: 'a>(ed: &'a Editor, pal: Pal, size: f32, on: impl Fn(EdMsg) -> M + 'a) -> EditorView<'a, M> { EditorView { ed, pal, size, on: Box::new(on) } }

#[derive(Default)]
struct State { cw: f32, lh: f32, focused: bool, dragging: bool, mods: keyboard::Modifiers, last_click: Option<(std::time::Instant, Pos)>, last_dims: (usize, usize) }

impl<'a, M> EditorView<'a, M> {
    fn lh(&self) -> f32 { (self.size * 1.45).round() }
    fn text_x(&self, b: Rectangle) -> f32 { b.x + GUTTER + PAD }
    /// Zeilenposition unter einem Mauspunkt
    fn pos_at(&self, st: &State, b: Rectangle, p: Point) -> Pos {
        let ed = self.ed; let row = (((p.y - b.y) / st.lh).floor() + ed.scroll_row.floor()).max(0.0) as usize;
        // der Editor ist unveränderlich (&), daher eine lokale Näherung ohne Cache: bei Umbruch/Falten über die Zeilenliste suchen
        let (line, sub) = rows_lookup(ed, row);
        let vis = ((p.x - self.text_x(b)) / st.cw).round().max(0.0) as usize + if ed.wrap { sub * ed.wrap_cols } else { ed.hscroll };
        Pos { line, col: col_at_vis(&ed.lines[line], vis) }
    }
}

/// Anzeigezeile -> (Zeile, Teilzeile) ohne `&mut` (nutzt dieselbe Logik wie `Editor::line_of_row`)
fn rows_lookup(ed: &Editor, row: usize) -> (usize, usize) {
    if !ed.wrap && ed.folds.is_empty() { return (row.min(ed.lines.len() - 1), 0) }
    let mut acc = 0;
    for l in 0..ed.lines.len() {
        if ed.is_hidden(l) { continue }
        let n = if ed.wrap { (vis_len(&ed.lines[l]).max(1) + ed.wrap_cols - 1) / ed.wrap_cols.max(1) } else { 1 };
        if row < acc + n { return (l, row - acc) }
        acc += n;
    }
    (ed.lines.len() - 1, 0)
}

impl<'a, M, R> Widget<M, iced::Theme, R> for EditorView<'a, M>
where R: text::Renderer<Font = Font> {
    fn tag(&self) -> tree::Tag { tree::Tag::of::<State>() }
    fn state(&self) -> tree::State { tree::State::new(State::default()) }
    fn size(&self) -> Size<Length> { Size::new(Length::Fill, Length::Fill) }

    fn layout(&mut self, tree: &mut Tree, _r: &R, limits: &layout::Limits) -> layout::Node {
        let st = tree.state.downcast_mut::<State>();
        st.lh = self.lh();
        // Zeichenbreite des Festschrittfonts messen (zehn Zeichen)
        let p = R::Paragraph::with_text(Text { content: "0000000000", bounds: Size::new(f32::INFINITY, f32::INFINITY), size: Pixels(self.size), line_height: text::LineHeight::Absolute(Pixels(st.lh)), font: Font::MONOSPACE, align_x: text::Alignment::Default, align_y: alignment::Vertical::Top, shaping: text::Shaping::Basic, wrapping: text::Wrapping::None });
        st.cw = p.min_bounds().width / 10.0;
        layout::Node::new(limits.max())
    }

    fn draw(&self, tree: &Tree, r: &mut R, _t: &iced::Theme, _s: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        let t0 = std::time::Instant::now();
        let st = tree.state.downcast_ref::<State>(); let b = layout.bounds(); let pal = self.pal; let ed = self.ed;
        let (cw, lh) = (st.cw, st.lh);
        let Some(clip) = b.intersection(viewport) else { return };
        let q = |r: &mut R, bounds: Rectangle, c: Color| r.fill_quad(Quad { bounds, ..Quad::default() }, c);
        let txt = |r: &mut R, s: &str, x: f32, y: f32, c: Color| {
            r.fill_text(Text { content: s.to_string(), bounds: Size::new(f32::INFINITY, lh), size: Pixels(self.size), line_height: text::LineHeight::Absolute(Pixels(lh)), font: Font::MONOSPACE, align_x: text::Alignment::Default, align_y: alignment::Vertical::Top, shaping: text::Shaping::Basic, wrapping: text::Wrapping::None }, Point::new(x, y), c, clip);
        };
        q(r, b, pal.editor);
        q(r, Rectangle { width: GUTTER, ..b }, pal.panel);
        let tx = self.text_x(b); let rows = (b.height / lh).ceil() as usize + 1;
        let first = ed.scroll_row.floor() as usize; let frac = (ed.scroll_row - ed.scroll_row.floor()) * lh;
        let sel = ed.selection(); let cols = ((b.width - GUTTER - 2.0 * PAD) / cw) as usize;
        let (mut line, mut sub) = rows_lookup(ed, first);
        let mut cursor_xy: Option<(f32, f32)> = None;
        for i in 0..rows {
            if line >= ed.lines.len() { break }
            let y = b.y + i as f32 * lh - frac;
            let text = &ed.lines[line];
            // Zeilenhintergründe
            if ed.debug_line == Some(line) { q(r, Rectangle { x: b.x + GUTTER, y, width: b.width - GUTTER, height: lh }, Color { a: 0.28, ..pal.warn }) }
            else if ed.errors.contains_key(&line) { q(r, Rectangle { x: b.x + GUTTER, y, width: b.width - GUTTER, height: lh }, Color { a: 0.16, ..pal.err }) }
            else if ed.warns.contains_key(&line) { q(r, Rectangle { x: b.x + GUTTER, y, width: b.width - GUTTER, height: lh }, Color { a: 0.14, ..pal.warn }) }
            else if ed.cursor.line == line && sel.is_none() { q(r, Rectangle { x: b.x + GUTTER, y, width: b.width - GUTTER, height: lh }, Color { a: 0.45, ..pal.sel }) }
            // Zellen der ganzen Zeile (Tabulatoren aufgelöst) mit Art
            let toks = tokens(text); let mut cells: Vec<(char, Kind)> = Vec::with_capacity(text.len() + 8); let mut v = 0usize;
            let lo = if ed.wrap { sub * ed.wrap_cols } else { ed.hscroll }; let hi = lo + if ed.wrap { ed.wrap_cols } else { cols + 2 };
            for (ci, ch) in text.chars().enumerate() {
                let kind = toks.iter().find(|(s, e, _)| ci >= *s && ci < *e).map(|t| t.2).unwrap_or(Kind::Plain);
                if ch == '\t' { let n = (v / TAB + 1) * TAB - v; for _ in 0..n { if v >= lo && v < hi { cells.push((' ', kind)) } v += 1 } } else { if v >= lo && v < hi { cells.push((ch, kind)) } v += 1 }
                if v >= hi { break }
            }
            // Auswahl
            if let Some((a, e)) = sel { if line >= a.line && line <= e.line {
                let from = if line == a.line { vis_col(text, a.col) } else { 0 }; let to = if line == e.line { vis_col(text, e.col) } else { vis_len(text) + 1 };
                let (f, t) = (from.max(lo), to.min(hi)); if t > f { q(r, Rectangle { x: tx + (f - lo) as f32 * cw, y, width: (t - f) as f32 * cw, height: lh }, pal.sel) } } }
            // Text in Läufen gleicher Art
            let mut i0 = 0;
            while i0 < cells.len() { let k = cells[i0].1; let mut j = i0; while j < cells.len() && cells[j].1 == k { j += 1 } let s: String = cells[i0..j].iter().map(|c| c.0).collect(); if !s.trim().is_empty() { txt(r, &s, tx + i0 as f32 * cw, y, pal.kind(k)) } i0 = j; }
            // Fehlerwelle
            if (ed.errors.contains_key(&line) || ed.warns.contains_key(&line)) && sub == 0 {
                let wc = if ed.errors.contains_key(&line) { pal.err } else { pal.warn };
                let w = (vis_len(text).saturating_sub(lo)).min(cells.len().max(1)) as f32 * cw; let mut x = 0.0; let mut up = false;
                while x < w.min(600.0) { q(r, Rectangle { x: tx + x, y: y + lh - 4.0 + if up { 0.0 } else { 2.0 }, width: 3.0, height: 1.5 }, wc); x += 3.0; up = !up }
            }
            // Cursor
            if ed.cursor.line == line {
                let v = vis_col(text, ed.cursor.col); let rs = if ed.wrap { v / ed.wrap_cols.max(1) } else { 0 };
                if rs == sub { let vc = if ed.wrap { v % ed.wrap_cols.max(1) } else { v.saturating_sub(ed.hscroll) }; cursor_xy = Some((tx + vc as f32 * cw, y));
                    if st.focused { q(r, Rectangle { x: tx + vc as f32 * cw, y: y + 1.0, width: 2.0, height: lh - 2.0 }, pal.accent) }
                    if let Some(pre) = &ed.preedit { txt(r, pre, tx + vc as f32 * cw, y, pal.accent); q(r, Rectangle { x: tx + vc as f32 * cw, y: y + lh - 2.0, width: pre.chars().count() as f32 * cw, height: 1.0 }, pal.accent) }
                }
            }
            // Rand: Zeilennummer, Haltepunkt, Faltmarke (nur in der ersten Anzeigezeile)
            if sub == 0 {
                let num = format!("{}", line + 1); txt(r, &num, b.x + GUTTER - 10.0 - num.len() as f32 * cw - 14.0, y, if ed.cursor.line == line { pal.text } else { pal.mute });
                if ed.breakpoints.contains(&line) { r.fill_quad(Quad { bounds: Rectangle { x: b.x + 6.0, y: y + (lh - 11.0) / 2.0, width: 11.0, height: 11.0 }, border: Border { radius: 5.5.into(), ..Border::default() }, ..Quad::default() }, pal.err) }
                if let Some(f) = ed.folds.get(&line) { let _ = f; txt(r, "▸", b.x + GUTTER - 16.0, y, pal.mute) } else if ed.lines[line].ends_with(':') && !ed.lines[line].starts_with(char::is_whitespace) && ed.fold_range(line).is_some() { txt(r, "▾", b.x + GUTTER - 16.0, y, pal.mute) }
            }
            // nächste Anzeigezeile
            let n = if ed.wrap { (vis_len(text).max(1) + ed.wrap_cols - 1) / ed.wrap_cols.max(1) } else { 1 };
            if sub + 1 < n { sub += 1 } else { sub = 0; line += 1; while line < ed.lines.len() && ed.is_hidden(line) { line += 1 } }
        }
        q(r, Rectangle { x: b.x + GUTTER, width: 1.0, ..b }, pal.line);
        // Vervollständigungsliste
        if let (Some(c), Some((cx, cy))) = (&ed.completion, cursor_xy) { r.with_layer(clip, |r| {
            let w = 230.0; let h = c.items.len() as f32 * (lh + 2.0) + 8.0; let x0 = cx.min(b.x + b.width - w - 4.0); let y0 = cy + lh + 2.0;
            r.fill_quad(Quad { bounds: Rectangle { x: x0, y: y0, width: w, height: h }, border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, ..Quad::default() }, pal.panel2);
            for (i, it) in c.items.iter().enumerate() { let yy = y0 + 4.0 + i as f32 * (lh + 2.0); if i == c.sel { q(r, Rectangle { x: x0 + 3.0, y: yy, width: w - 6.0, height: lh }, pal.sel) } txt(r, it, x0 + 10.0, yy, pal.text) }
        }) }
        let _ = cursor; DRAW_NS.fetch_add(t0.elapsed().as_nanos() as u64, Ordering::Relaxed); DRAW_N.fetch_add(1, Ordering::Relaxed);
    }

    fn update(&mut self, tree: &mut Tree, event: &Event, layout: Layout<'_>, cursor: mouse::Cursor, _r: &R, clipboard: &mut dyn Clipboard, shell: &mut Shell<'_, M>, _vp: &Rectangle) {
        let st = tree.state.downcast_mut::<State>(); let b = layout.bounds(); let ed = self.ed;
        match event {
            Event::Window(window::Event::RedrawRequested(_)) => {
                let cols = (((b.width - GUTTER - 2.0 * PAD) / st.cw).floor() as usize).max(10); let rows = ((b.height / st.lh).floor() as usize).max(2);
                if (cols, rows) != (ed.wrap_cols, ed.view_rows) && (cols, rows) != st.last_dims { st.last_dims = (cols, rows); shell.publish((self.on)(EdMsg::Resize(cols, rows))) }
                if st.focused {
                    // Cursorrechteck für das IME-Fenster
                    let v = vis_col(&ed.lines[ed.cursor.line], ed.cursor.col);
                    let rect = Rectangle { x: self.text_x(b) + (v.saturating_sub(ed.hscroll)) as f32 * st.cw, y: b.y + (ed.cursor.line as f32 - ed.scroll_row).max(0.0) * st.lh, width: 2.0, height: st.lh };
                    shell.request_input_method(&InputMethod::Enabled { cursor: rect, purpose: input_method::Purpose::Normal, preedit: ed.preedit.as_deref().map(|c| input_method::Preedit { content: c, selection: None, text_size: Some(Pixels(self.size)) }) });
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(p) = cursor.position_in(b) {
                    st.focused = true; shell.capture_event(); let abs = Point::new(b.x + p.x, b.y + p.y);
                    if p.x < GUTTER { let (line, _) = rows_lookup(ed, ((p.y / st.lh).floor() + ed.scroll_row.floor()) as usize); if p.x > GUTTER - 22.0 { shell.publish((self.on)(EdMsg::Fold(line))) } else { shell.publish((self.on)(EdMsg::Breakpoint(line))) } }
                    else { let pos = self.pos_at(st, b, abs); let now = std::time::Instant::now();
                        // Doppelklick: Wort markieren (über Home/End-Näherung: Klick + Auswahl bis Wortende)
                        let dbl = st.last_click.map_or(false, |(t, pp)| now.duration_since(t).as_millis() < 350 && pp == pos);
                        st.last_click = Some((now, pos)); st.dragging = true;
                        shell.publish((self.on)(EdMsg::Click(pos, st.mods.shift(), dbl))); }
                } else { st.focused = false; }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if st.dragging => { let pos = self.pos_at(st, b, *position); shell.publish((self.on)(EdMsg::Drag(pos))); shell.capture_event() }
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => { st.mods = *m }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => { st.dragging = false }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(b) => {
                let y = match delta { mouse::ScrollDelta::Lines { y, .. } => *y * 3.0, mouse::ScrollDelta::Pixels { y, .. } => *y / st.lh };
                shell.publish((self.on)(EdMsg::Scroll(-y))); shell.capture_event()
            }
            Event::InputMethod(ev) if st.focused => match ev {
                input_method::Event::Preedit(s, _) => { shell.publish((self.on)(EdMsg::Preedit(if s.is_empty() { None } else { Some(s.clone()) }))); shell.request_redraw() }
                input_method::Event::Commit(s) => { shell.publish((self.on)(EdMsg::Preedit(None))); shell.publish((self.on)(EdMsg::Insert(s.clone()))); shell.capture_event() }
                _ => {}
            },
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, text, .. }) if st.focused => {
                let cmd = modifiers.command(); let shift = modifiers.shift(); let mut m = |e: EdMsg| shell.publish((self.on)(e));
                let comp = ed.completion.is_some();
                let handled = match key {
                    Key::Named(Named::ArrowUp) if comp => { m(EdMsg::CompletionMove(-1)); true }
                    Key::Named(Named::ArrowDown) if comp => { m(EdMsg::CompletionMove(1)); true }
                    Key::Named(Named::Enter) | Key::Named(Named::Tab) if comp => { m(EdMsg::CompletionAccept); true }
                    Key::Named(Named::Escape) => { m(EdMsg::CompletionClose); true }
                    Key::Named(Named::ArrowUp) => { m(EdMsg::Move { dx: 0, dy: -1, extend: shift }); true }
                    Key::Named(Named::ArrowDown) => { m(EdMsg::Move { dx: 0, dy: 1, extend: shift }); true }
                    Key::Named(Named::ArrowLeft) if cmd => { m(EdMsg::Home(shift)); true }
                    Key::Named(Named::ArrowRight) if cmd => { m(EdMsg::End(shift)); true }
                    Key::Named(Named::ArrowLeft) => { m(EdMsg::Move { dx: -1, dy: 0, extend: shift }); true }
                    Key::Named(Named::ArrowRight) => { m(EdMsg::Move { dx: 1, dy: 0, extend: shift }); true }
                    Key::Named(Named::Home) => { m(EdMsg::Home(shift)); true }
                    Key::Named(Named::End) => { m(EdMsg::End(shift)); true }
                    Key::Named(Named::PageUp) => { m(EdMsg::Page(-1, shift)); true }
                    Key::Named(Named::PageDown) => { m(EdMsg::Page(1, shift)); true }
                    Key::Named(Named::Backspace) => { m(EdMsg::Backspace); true }
                    Key::Named(Named::Delete) => { m(EdMsg::Delete); true }
                    Key::Named(Named::Enter) => { m(EdMsg::Enter); true }
                    Key::Named(Named::Tab) => { m(EdMsg::Tab); true }
                    Key::Character(c) if cmd => match c.as_str() {
                        "a" => { m(EdMsg::SelectAll); true }
                        "z" => { m(if shift { EdMsg::Redo } else { EdMsg::Undo }); true }
                        "y" => { m(EdMsg::Redo); true }
                        "c" => { if let Some(s) = ed.selected_text() { clipboard.write(clipboard::Kind::Standard, s) } true }
                        "x" => { if let Some(s) = ed.selected_text() { clipboard.write(clipboard::Kind::Standard, s); m(EdMsg::Backspace) } true }
                        "v" => { if let Some(s) = clipboard.read(clipboard::Kind::Standard) { m(EdMsg::Insert(s.replace("\r\n", "\n"))) } true }
                        _ => false,
                    },
                    _ => false,
                };
                if handled { shell.capture_event(); shell.request_redraw(); }
                else if !cmd && !modifiers.control() { if let Some(t) = text { if !t.is_empty() && !t.chars().any(|c| c.is_control()) { shell.publish((self.on)(EdMsg::Insert(t.to_string()))); shell.capture_event(); shell.request_redraw() } } }
            }
            _ => {}
        }
    }
    fn mouse_interaction(&self, tree: &Tree, layout: Layout<'_>, cursor: mouse::Cursor, _vp: &Rectangle, _r: &R) -> mouse::Interaction {
        let _ = tree;
        match cursor.position_in(layout.bounds()) { Some(p) if p.x > GUTTER => mouse::Interaction::Text, Some(_) => mouse::Interaction::Pointer, None => mouse::Interaction::None }
    }
}
impl<'a, M: 'a, R> From<EditorView<'a, M>> for Element<'a, M, iced::Theme, R> where R: text::Renderer<Font = Font> + 'a { fn from(v: EditorView<'a, M>) -> Self { Element::new(v) } }
