// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: Editor als eigenes Iced-Widget (Trait `iced::advanced::Widget`). Festes Zeichenraster, eigener Rand mit Zeilennummern,
//! Haltepunkten und Faltmarken, Auswahl, Fehlerwellen, Zeilenfarben, Vervollständigungsliste, IME, Mausauswahl.
use crate::core::{col_at_vis, vis_col, vis_len, Editor, Pos, TAB};
use crate::highlight::{tokens, Kind};
use vc_gui_ui::theme::Pal;
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
const SB: f32 = 12.0;

#[derive(Debug, Clone)]
pub enum EdMsg {
    Insert(String), Backspace, Delete, Enter, Tab,
    Move { dx: i32, dy: i32, extend: bool }, Home(bool), End(bool), Page(i32, bool),
    SelectAll, Undo, Redo, Click(Pos, bool, bool), Drag(Pos), Breakpoint(usize), Fold(usize), Scroll(f32),
    Resize(usize, usize), CompletionMove(i32), CompletionAccept, CompletionClose, Preedit(Option<String>), ToggleWrap, Hover(Option<Pos>), HoverXY(Point), Kontext(Pos, Point), HScroll(usize), VScroll(f32),
    /// Beim Ziehen einer Datei aus dem Projektbaum: Zeilengrenze unter dem Zeiger (für die Einfügemarke) bzw. losgelassen
    AblageUeber(Option<usize>), Ablegen(usize),
}

pub struct EditorView<'a, M> { ed: &'a Editor, pal: Pal, size: f32, on: Box<dyn Fn(EdMsg) -> M + 'a>, fokus: u64, aktiv: bool, ziehen: bool, /** Buchstaben, die mit Alt das Menü öffnen (Menüleiste im Fenster): der Editor fügt dafür nichts ein und fängt die Taste nicht ab */ alt_menue: Vec<char> }
pub fn editor_view<'a, M: 'a>(ed: &'a Editor, pal: Pal, size: f32, on: impl Fn(EdMsg) -> M + 'a) -> EditorView<'a, M> { EditorView { ed, pal, size, on: Box::new(on), fokus: 0, aktiv: true, ziehen: false, alt_menue: vec![] } }

const HOVER_VERZOEGERUNG: std::time::Duration = std::time::Duration::from_millis(450);
#[derive(Default)]
struct State { abl: Option<usize>, cw: f32, lh: f32, focused: bool, dragging: bool, mods: keyboard::Modifiers, last_click: Option<(std::time::Instant, Pos)>, last_dims: (usize, usize), hover: Option<(Pos, std::time::Instant, bool)>, sb_drag: Option<f32>, vb_drag: Option<f32>, last_fokus: u64 }

impl<'a, M> EditorView<'a, M> {
    /// Jede Erhöhung dieser Zahl gibt dem Editor die Tastatur zurück (z. B. nach dem Einfügen aus einem Seitenpanel)
    pub fn fokus(mut self, n: u64) -> Self { self.fokus = n; self }
    /// Darf der Editor die Tastatur haben? Das Modell hat **genau einen Fokusbesitzer**; ist es ein anderes Feld (Suchen, Palette, Dialog …), verwirft der Editor Tasten-, Text- und IME-Ereignisse.
    pub fn aktiv(mut self, a: bool) -> Self { self.aktiv = a; self }
    /// Läuft gerade ein Ziehen aus dem Projektbaum? Dann meldet der Editor die Zeilengrenze unter dem Zeiger und das Loslassen.
    pub fn ziehen(mut self, z: bool) -> Self { self.ziehen = z; self }
    pub fn alt_menue(mut self, b: Vec<char>) -> Self { self.alt_menue = b; self }
    /// Zeilengrenze (0 = vor der ersten Zeile, n = hinter der letzten) unter dem Punkt: obere Zeilenhälfte davor, untere dahinter
    fn grenze_at(&self, st: &State, b: Rectangle, p: Point) -> usize {
        let pos = self.pos_at(st, b, p); let f = ((p.y - b.y) / st.lh.max(1.0)).fract(); if f < 0.5 { pos.line } else { pos.line + 1 }
    }
    fn lh(&self) -> f32 { (self.size * 1.45).round() }
    fn text_x(&self, b: Rectangle) -> f32 { b.x + GUTTER + PAD }
    /// Waagerechte Scrollleiste: (Schiene, Griff, Verschiebungsbereich in Spalten); nur ohne Umbruch und wenn eine Zeile breiter ist als der Sichtbereich
    fn hbar(&self, b: Rectangle) -> Option<(Rectangle, Rectangle, usize)> {
        let ed = self.ed; let max = ed.hscroll_max(); if max == 0 { return None }
        let track = Rectangle { x: b.x + GUTTER, y: b.y + b.height - SB, width: b.width - GUTTER, height: SB };
        let total = (max + ed.wrap_cols) as f32; let tw = (track.width * ed.wrap_cols as f32 / total).max(28.0).min(track.width.max(1.0));
        let x = track.x + (track.width - tw) * (ed.hscroll as f32 / max as f32);
        Some((track, Rectangle { x, y: track.y + 2.0, width: tw, height: SB - 4.0 }, max))
    }
    /// Senkrechte Scrollleiste am rechten Rand: (Schiene, Griff, größte Zeile); nur wenn nicht alles sichtbar ist
    fn vbar(&self, b: Rectangle) -> Option<(Rectangle, Rectangle, f32)> {
        let ed = self.ed; let total = ed.rows_total_ro() as f32; let view = ed.view_rows as f32; if total <= view { return None }
        let hoehe = b.height - if self.hbar(b).is_some() { SB } else { 0.0 };
        let track = Rectangle { x: b.x + b.width - SB, y: b.y, width: SB, height: hoehe };
        let max = (total - 3.0).max(1.0); let th = (hoehe * view / total).max(28.0).min(hoehe.max(1.0));
        let y = track.y + (hoehe - th) * (ed.scroll_row / max).clamp(0.0, 1.0);
        Some((track, Rectangle { x: track.x + 2.0, y, width: SB - 4.0, height: th }, max))
    }
    fn vwert(&self, b: Rectangle, griff: f32, py: f32) -> f32 {
        let Some((track, thumb, max)) = self.vbar(b) else { return 0.0 };
        let frei = (track.height - thumb.height).max(1.0); ((py - griff - track.y) / frei).clamp(0.0, 1.0) * max
    }
    /// Verschiebung (Spalten) zu einer Mausposition auf der Schiene; `griff`: Abstand des Mauspunkts vom linken Griffrand
    fn hwert(&self, b: Rectangle, griff: f32, px: f32) -> usize {
        let Some((track, thumb, max)) = self.hbar(b) else { return 0 };
        let frei = (track.width - thumb.width).max(1.0); (((px - griff - track.x) / frei).clamp(0.0, 1.0) * max as f32).round() as usize
    }
    /// Zeilenposition unter einem Mauspunkt
    /// Zeichenzelle UNTER dem Mauszeiger (nicht die nächste Zeichengrenze wie beim Klicken); `None`, wenn dort kein Zeichen steht
    fn zelle_unter(&self, st: &State, b: Rectangle, p: Point) -> Option<Pos> {
        let ed = self.ed; if p.y < b.y { return None }
        let row = (((p.y - b.y) / st.lh).floor() + ed.scroll_row.floor()).max(0.0) as usize;
        let (line, sub) = rows_lookup(ed, row);
        let vis = ((p.x - self.text_x(b)) / st.cw).floor();
        if vis < 0.0 { return None }
        let vis = vis as usize + if ed.wrap { sub * ed.wrap_cols } else { ed.hscroll };
        let col = col_at_vis(&ed.lines[line], vis);
        (col < ed.lines[line].chars().count() && vis < vis_len(&ed.lines[line])).then_some(Pos { line, col })
    }
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
        let p = R::Paragraph::with_text(Text { content: "0000000000", bounds: Size::new(f32::INFINITY, f32::INFINITY), size: Pixels(self.size), line_height: text::LineHeight::Absolute(Pixels(st.lh)), font: vc_gui_ui::fonts::MONO, align_x: text::Alignment::Default, align_y: alignment::Vertical::Top, shaping: text::Shaping::Basic, wrapping: text::Wrapping::None });
        st.cw = p.min_bounds().width / 10.0;
        layout::Node::new(limits.max())
    }

    fn draw(&self, tree: &Tree, r: &mut R, _t: &iced::Theme, _s: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        let t0 = std::time::Instant::now();
        let st = tree.state.downcast_ref::<State>(); let b = layout.bounds(); let pal = self.pal; let ed = self.ed;
        let (cw, lh) = (st.cw, st.lh);
        let Some(clip) = b.intersection(viewport) else { return };
        // Alles innerhalb der eigenen Grenzen halten: die letzte, nur zum Teil sichtbare Zeile (und die Zeile darunter) darf nicht in die Nachbarflächen
        // (Schublade, Statusleiste) hineinragen. Die Ebene beschneidet auf `clip`; der Text meldet die größere Fläche `viewport` als eigenen Beschnitt, denn der
        // Software-Renderer schneidet Text nur dann an der Ebene ab, wenn dessen eigener Beschnitt über die Ebene hinausragt.
        r.with_layer(clip, |r| {
        let q = |r: &mut R, bounds: Rectangle, c: Color| r.fill_quad(Quad { bounds, ..Quad::default() }, c);
        let txt = |r: &mut R, s: &str, x: f32, y: f32, c: Color| {
            r.fill_text(Text { content: s.to_string(), bounds: Size::new(f32::INFINITY, lh), size: Pixels(self.size), line_height: text::LineHeight::Absolute(Pixels(lh)), font: vc_gui_ui::fonts::MONO, align_x: text::Alignment::Default, align_y: alignment::Vertical::Top, shaping: text::Shaping::Basic, wrapping: text::Wrapping::None }, Point::new(x, y), c, *viewport);
        };
        q(r, b, pal.editor);
        q(r, Rectangle { width: GUTTER, ..b }, pal.panel);
        let tx = self.text_x(b); let rows = (b.height / lh).ceil() as usize + 1;
        let first = ed.scroll_row.floor() as usize; let frac = (ed.scroll_row - ed.scroll_row.floor()) * lh;
        let sel = ed.selection(); let cols = ((b.width - GUTTER - 2.0 * PAD) / cw) as usize;
        let (mut line, mut sub) = rows_lookup(ed, first);
        let mut cursor_xy: Option<(f32, f32)> = None;
        for i in 0..rows {
            let y = b.y + i as f32 * lh - frac;
            // Steht das Fenster so, dass die letzte Zeile nur halb Platz hätte, bleibt sie weg (die Seitengröße der Tasten und des Cursors rechnet ohnehin mit ganzen Zeilen);
            // beim sanften Scrollen (Bruchteil ungleich 0) gehören angeschnittene Zeilen am Rand dazu
            if !zeile_passt(frac, y, lh, b.y + b.height) { break }
            if line >= ed.lines.len() { if ed.einfuegemarke == Some(ed.lines.len()) { q(r, Rectangle { x: b.x + GUTTER, y: y - 1.0, width: b.width - GUTTER, height: 2.0 }, pal.accent) } break }
            if sub == 0 && ed.einfuegemarke == Some(line) { q(r, Rectangle { x: b.x + GUTTER, y: y - 1.0, width: b.width - GUTTER, height: 2.0 }, pal.accent) }
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
            if let Some(ms) = ed.match_lines.get(&line) { for (c0, c1, cur) in ms { let (f, t) = (vis_col(text, *c0).max(lo), vis_col(text, *c1).min(hi)); if t > f { q(r, Rectangle { x: tx + (f - lo) as f32 * cw, y, width: (t - f) as f32 * cw, height: lh }, Color { a: if *cur { 0.55 } else { 0.28 }, ..pal.warn }) } } }
            // Text in Läufen gleicher Art
            let mut i0 = 0;
            while i0 < cells.len() { let k = cells[i0].1; let mut j = i0; while j < cells.len() && cells[j].1 == k { j += 1 } let s: String = cells[i0..j].iter().map(|c| c.0).collect(); if !s.trim().is_empty() { txt(r, &s, tx + i0 as f32 * cw, y, pal.kind(k)) } i0 = j; }
            // Fehlerwelle
            if (ed.errors.contains_key(&line) || ed.warns.contains_key(&line) || ed.hints.contains_key(&line)) && sub == 0 {
                let wc = if ed.errors.contains_key(&line) { pal.err } else if ed.warns.contains_key(&line) { pal.warn } else { pal.mute };
                let hint_only = !ed.errors.contains_key(&line) && !ed.warns.contains_key(&line);
                let w = (vis_len(text).saturating_sub(lo)).min(cells.len().max(1)) as f32 * cw; let mut x = 0.0; let mut up = false;
                if hint_only { while x < w.min(600.0) { q(r, Rectangle { x: tx + x, y: y + lh - 3.0, width: 1.5, height: 1.5 }, wc); x += 4.0 } }
                else { while x < w.min(600.0) { q(r, Rectangle { x: tx + x, y: y + lh - 4.0 + if up { 0.0 } else { 2.0 }, width: 3.0, height: 1.5 }, wc); x += 3.0; up = !up } }
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
                if let Some(f) = ed.folds.get(&line) { let _ = f; txt(r, "▸", b.x + GUTTER - 16.0, y, pal.mute) } else if ed.fold_range(line).is_some() { txt(r, "▾", b.x + GUTTER - 16.0, y, pal.mute) }
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
            for (i, it) in c.items.iter().enumerate() { let yy = y0 + 4.0 + i as f32 * (lh + 2.0); if i == c.sel { q(r, Rectangle { x: x0 + 3.0, y: yy, width: w - 6.0, height: lh }, pal.sel) } txt(r, it, x0 + 10.0, yy, pal.text); if let Some(d) = c.details.get(i).filter(|d| !d.is_empty()) { let d: String = d.chars().take(((w - 20.0) / cw) as usize - it.chars().count() - 1).collect(); txt(r, &d, x0 + 10.0 + (it.chars().count() as f32 + 1.0) * cw, yy, pal.mute) } }
        }) }
        if let Some((track, thumb, _)) = self.vbar(b) { q(r, track, Color { a: 0.25, ..pal.panel2 }); r.fill_quad(Quad { bounds: thumb, border: Border { radius: 3.0.into(), ..Border::default() }, ..Quad::default() }, Color { a: if st.vb_drag.is_some() { 0.8 } else { 0.45 }, ..pal.mute }) }
        if let Some((track, thumb, _)) = self.hbar(b) { q(r, track, Color { a: 0.35, ..pal.panel2 }); r.fill_quad(Quad { bounds: thumb, border: Border { radius: 3.0.into(), ..Border::default() }, ..Quad::default() }, Color { a: if st.sb_drag.is_some() { 0.8 } else { 0.5 }, ..pal.mute }) }
        });
        let _ = cursor; DRAW_NS.fetch_add(t0.elapsed().as_nanos() as u64, Ordering::Relaxed); DRAW_N.fetch_add(1, Ordering::Relaxed);
    }

    fn update(&mut self, tree: &mut Tree, event: &Event, layout: Layout<'_>, cursor: mouse::Cursor, _r: &R, clipboard: &mut dyn Clipboard, shell: &mut Shell<'_, M>, _vp: &Rectangle) {
        let st = tree.state.downcast_mut::<State>(); let b = layout.bounds(); let ed = self.ed;
        if !self.aktiv && !matches!(event, Event::Mouse(_) | Event::Touch(_)) { st.focused = false; st.last_fokus = self.fokus }
        match event {
            Event::Window(window::Event::RedrawRequested(_)) => {
                if self.fokus != st.last_fokus { st.last_fokus = self.fokus; st.focused = true }
                if let Some((p, t, false)) = st.hover { if t.elapsed() >= HOVER_VERZOEGERUNG { st.hover = Some((p, t, true)); { if let Some(mp) = cursor.position() { shell.publish((self.on)(EdMsg::HoverXY(mp))) } shell.publish((self.on)(EdMsg::Hover(Some(p)))) } } else { shell.request_redraw_at(t + HOVER_VERZOEGERUNG) } }
                let cols = (((b.width - GUTTER - 2.0 * PAD) / st.cw).floor() as usize).max(10); let rows = ((b.height / st.lh).floor() as usize).max(2);
                if (cols, rows) != (ed.wrap_cols, ed.view_rows) && (cols, rows) != st.last_dims { st.last_dims = (cols, rows); shell.publish((self.on)(EdMsg::Resize(cols, rows))) }
                if st.focused {
                    // Cursorrechteck für das IME-Fenster
                    let v = vis_col(&ed.lines[ed.cursor.line], ed.cursor.col);
                    let rect = Rectangle { x: self.text_x(b) + (v.saturating_sub(ed.hscroll)) as f32 * st.cw, y: b.y + (ed.cursor.line as f32 - ed.scroll_row).max(0.0) * st.lh, width: 2.0, height: st.lh };
                    shell.request_input_method(&InputMethod::Enabled { cursor: rect, purpose: input_method::Purpose::Normal, preedit: ed.preedit.as_deref().map(|c| input_method::Preedit { content: c, selection: None, text_size: Some(Pixels(self.size)) }) });
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if self.vbar(b).zip(cursor.position()).is_some_and(|((t, _, _), p)| t.contains(p)) => {
                let p = cursor.position().unwrap(); let (_, thumb, _) = self.vbar(b).unwrap();
                let griff = if thumb.contains(p) { p.y - thumb.y } else { thumb.height / 2.0 }; st.vb_drag = Some(griff);
                shell.publish((self.on)(EdMsg::VScroll(self.vwert(b, griff, p.y)))); shell.capture_event()
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if st.vb_drag.is_some() => { shell.publish((self.on)(EdMsg::VScroll(self.vwert(b, st.vb_drag.unwrap_or(0.0), position.y)))); shell.capture_event() }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if self.hbar(b).zip(cursor.position()).is_some_and(|((t, _, _), p)| t.contains(p)) => {
                let p = cursor.position().unwrap(); let (_, thumb, _) = self.hbar(b).unwrap();
                let griff = if thumb.contains(p) { p.x - thumb.x } else { thumb.width / 2.0 }; st.sb_drag = Some(griff);
                shell.publish((self.on)(EdMsg::HScroll(self.hwert(b, griff, p.x)))); shell.capture_event()
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if st.sb_drag.is_some() => { shell.publish((self.on)(EdMsg::HScroll(self.hwert(b, st.sb_drag.unwrap_or(0.0), position.x)))); shell.capture_event() }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) if cursor.position_in(b).is_some_and(|p| p.x > GUTTER) => {
                let abs = cursor.position().unwrap(); st.focused = true; let pos = self.pos_at(st, b, abs); shell.publish((self.on)(EdMsg::Kontext(pos, abs))); shell.capture_event()
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
            Event::Mouse(mouse::Event::CursorMoved { position }) if self.ziehen => { let neu = b.contains(*position).then(|| self.grenze_at(st, b, *position)); if st.abl != neu { st.abl = neu; shell.publish((self.on)(EdMsg::AblageUeber(neu))) } }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if self.ziehen && cursor.position().is_some_and(|p| b.contains(p)) => { if let Some(p) = cursor.position() { st.abl = None; shell.publish((self.on)(EdMsg::Ablegen(self.grenze_at(st, b, p)))); shell.capture_event() } }
            Event::Mouse(mouse::Event::CursorMoved { position }) if st.dragging => { let pos = self.pos_at(st, b, *position); shell.publish((self.on)(EdMsg::Drag(pos))); shell.capture_event() }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let im_text = cursor.position_in(b).is_some_and(|p| p.x > GUTTER);
                if im_text {
                    match self.zelle_unter(st, b, *position) {
                        // Der Hinweis gilt dem Bereich unter der Maus: solange die Maus darin bleibt, bleibt er stehen
                        Some(z) if ed.hover.as_ref().is_some_and(|h| h.enthaelt(z)) => { st.hover = Some((z, std::time::Instant::now(), true)) }
                        Some(z) => { if st.hover.map(|h| h.0) != Some(z) { if ed.hover.is_some() { shell.publish((self.on)(EdMsg::Hover(None))) } let now = std::time::Instant::now(); st.hover = Some((z, now, false)); shell.request_redraw_at(now + HOVER_VERZOEGERUNG) } }
                        None => { st.hover = None; if ed.hover.is_some() { shell.publish((self.on)(EdMsg::Hover(None))) } }
                    }
                } else if st.hover.take().is_some() && ed.hover.is_some() { shell.publish((self.on)(EdMsg::Hover(None))) }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => { st.mods = *m }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => { st.dragging = false; st.sb_drag = None; st.vb_drag = None }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(b) => {
                st.hover = None; if self.ed.hover.is_some() { shell.publish((self.on)(EdMsg::Hover(None))) }
                let (x, y) = match delta { mouse::ScrollDelta::Lines { x, y } => (*x * 6.0, *y * 3.0), mouse::ScrollDelta::Pixels { x, y } => (*x / st.cw, *y / st.lh) };
                if x.abs() >= 0.5 && x.abs() > y.abs() && !self.ed.wrap { let neu = (self.ed.hscroll as f32 - x).round().max(0.0) as usize; shell.publish((self.on)(EdMsg::HScroll(neu))) } else { shell.publish((self.on)(EdMsg::Scroll(-y))) }
                shell.capture_event()
            }
            Event::InputMethod(ev) if st.focused => match ev {
                input_method::Event::Preedit(s, _) => { shell.publish((self.on)(EdMsg::Preedit(if s.is_empty() { None } else { Some(s.clone()) }))); shell.request_redraw() }
                input_method::Event::Commit(s) => { shell.publish((self.on)(EdMsg::Preedit(None))); shell.publish((self.on)(EdMsg::Insert(s.clone()))); shell.capture_event() }
                _ => {}
            },
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, text, .. }) if st.focused => {
                let cmd = modifiers.command(); let shift = modifiers.shift(); let mut m = |e: EdMsg| shell.publish((self.on)(e));
                let comp = ed.completion.is_some();
                // Kürzel der Befehlstabelle gehen **immer zuerst** an die Befehlstabelle: nur Tasten ohne Cmd/Strg/Alt (Umschalt erlaubt) sind Editor-Sache;
                // mit Cmd bleiben nur die editoreigenen Bearbeitungen (A, Z, Y, C, X, V) und Cmd+Pfeil links/rechts (Zeilenanfang/-ende) im Editor
                let befehl = (modifiers.control() || modifiers.alt() || cmd) && !matches!(key, Key::Character(_)) && !(cmd && !modifiers.alt() && !modifiers.control() && matches!(key, Key::Named(Named::ArrowLeft | Named::ArrowRight)));
                let handled = if befehl { false } else { match key {
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
                        "a" if !shift => { m(EdMsg::SelectAll); true }
                        "z" => { m(if shift { EdMsg::Redo } else { EdMsg::Undo }); true }
                        "y" if !shift => { m(EdMsg::Redo); true }
                        "c" if !shift => { if let Some(s) = ed.selected_text() { clipboard.write(clipboard::Kind::Standard, s) } true }
                        "x" if !shift => { if let Some(s) = ed.selected_text() { clipboard.write(clipboard::Kind::Standard, s); m(EdMsg::Backspace) } true }
                        "v" if !shift => { if let Some(s) = clipboard.read(clipboard::Kind::Standard) { m(EdMsg::Insert(s.replace("\r\n", "\n"))) } true }
                        _ => false,
                    },
                    _ => false,
                } };
                // Alt+Buchstabe gehört der Menüleiste (unter macOS liefert Alt+D sonst „∂“); AltGr (= Strg+Alt) und alle anderen Alt-Zeichen bleiben Text
                let fuer_menue = modifiers.alt() && !modifiers.control() && !cmd && !self.alt_menue.is_empty() && matches!(&key, Key::Character(c) if c.chars().next().is_some_and(|ch| self.alt_menue.contains(&ch.to_ascii_lowercase())));
                if handled { shell.capture_event(); shell.request_redraw(); }
                else if fuer_menue {}
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

/// Gehört die Zeile mit Oberkante `y` zur Anzeige? Eine nur halb Platz findende letzte Zeile bleibt weg (Seitengröße und Cursor rechnen mit ganzen Zeilen);
/// beim sanften Scrollen (Bruchteil `frac` ungleich 0) gehören angeschnittene Zeilen am Rand dazu.
fn zeile_passt(frac: f32, y: f32, lh: f32, unten: f32) -> bool { frac != 0.0 || y + lh <= unten + 0.5 }

#[cfg(test)]
mod zeilen_tests {
    use super::zeile_passt;
    #[test]
    fn halbe_letzte_zeile_bleibt_weg_ausser_beim_sanften_scrollen() {
        let (lh, unten) = (19.0, 100.0);          // 5 ganze Zeilen und ein Rest von 5 Punkten
        for i in 0..5 { assert!(zeile_passt(0.0, i as f32 * lh, lh, unten), "Zeile {i} passt ganz hinein"); }
        assert!(!zeile_passt(0.0, 5.0 * lh, lh, unten), "die sechste hätte nur 5 von 19 Punkten Platz");
        assert!(zeile_passt(0.0, 4.0 * lh + 24.0, lh, 4.0 * lh + 24.0 + lh), "genau passend");
        assert!(zeile_passt(0.4 * lh, 5.0 * lh, lh, unten), "sanftes Scrollen zeigt angeschnittene Zeilen");
    }
}
