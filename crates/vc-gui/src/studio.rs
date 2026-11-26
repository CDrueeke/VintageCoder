// SPDX-License-Identifier: GPL-3.0-or-later
//! Pixel-Studio (Stufe 3): Sprite/PMG und Zeichensatz in einem Dokument-Reiter. Das Modell ist `vc_ide_model::pixel::PixelDoc` (ohne Oberfläche);
//! hier stehen die Zeichenfläche (Canvas mit ganzzahligem Zoom, Raster, Onion Skin, Auswahl), die Werkzeugleiste und die Seitenspalten.
use super::*;
use vc_gui_ui::muster;
use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{text, tooltip};
use vc_ide_model::pixel::{AnimOp, PixelDoc, Tool};
use vc_media::sprite::AnimMode as sprite_mode;

/// Farbtabellen der Zielsysteme (PAL-Variante), ein Palettenindex ergibt eine Anzeigefarbe
/// Welche Videonorm-Palette das Studio zeigt: die des Projekts oder – nur zur Ansicht – die andere
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ansicht { #[default] Projekt, Pal, Ntsc }

/// Paletten aller Zielsysteme je Videonorm (`data/palettes`); Systeme ohne getrennte Tabellen haben nur eine
pub struct Farben { by: HashMap<(String, &'static str), vc_media::palette::TargetPalette>, mit_varianten: std::collections::HashSet<String>, standard: Option<vc_media::palette::TargetPalette> }
impl Farben {
    pub fn laden(data: &std::path::Path) -> Farben {
        let mut by = HashMap::new(); let mut mit = std::collections::HashSet::new();
        for t in vc_media::palette::known_targets(data) {
            let (p, n) = (vc_media::palette::palette_for(data, &t, "pal"), vc_media::palette::palette_for(data, &t, "ntsc"));
            if let (Ok(p), Ok(n)) = (&p, &n) { if p != n { mit.insert(t.clone()); } }
            if let Ok(p) = p { by.insert((t.clone(), "pal"), p); } if let Ok(n) = n { by.insert((t, "ntsc"), n); }
        }
        Farben { by, mit_varianten: mit, standard: vc_media::palette::palette_for(data, "", "pal").ok() }
    }
    fn tabelle(&self, ziel: &str, norm: &'static str) -> Option<&vc_media::palette::TargetPalette> { self.by.get(&(ziel.to_string(), norm)).or(self.standard.as_ref()) }
    /// Gibt es für das Ziel getrennte PAL- und NTSC-Tabellen? (sonst keine Anzeige/Umschaltung)
    pub fn hat_varianten(&self, ziel: &str) -> bool { self.mit_varianten.contains(ziel) }
    pub fn rgb(&self, ziel: &str, norm: &'static str, n: usize) -> [u8; 3] { self.tabelle(ziel, norm).and_then(|p| p.rgb(n)).unwrap_or([128, 128, 128]) }
    pub fn farbe(&self, ziel: &str, norm: &'static str, n: usize) -> Color { self.tabelle(ziel, norm).and_then(|p| p.rgb(n)).map_or(Color::from_rgb(0.5, 0.5, 0.5), |c| Color::from_rgb8(c[0], c[1], c[2])) }
    /// Alle Einträge des Ziels (Index, Farbe)
    pub fn eintraege(&self, ziel: &str, norm: &'static str) -> Vec<(u32, Color)> { self.tabelle(ziel, norm).map(|p| p.entries().into_iter().map(|(i, c)| (i, Color::from_rgb8(c[0], c[1], c[2]))).collect()).unwrap_or_default() }
}

/// Zustand eines geöffneten Pixel-Studios
pub struct PixelStudio { pub farb_slot: usize, pub mehr: bool, pub karten_zu: std::collections::BTreeSet<&'static str>, pub doc: PixelDoc, pub zoomansicht: vc_gui_ui::flaeche::Ansicht, pub raster: bool, pub zwiebel: bool, pub von: Option<(usize, usize)>, pub rechts: bool, pub ueber: Option<(usize, usize)>, pub ansicht: Ansicht, pub anim_t0: Option<std::time::Instant> }
impl PixelStudio {
    pub fn oeffnen(root: &std::path::Path, rel: &str, data: &std::path::Path) -> Result<PixelStudio, String> { Ok(PixelStudio { farb_slot: 0, mehr: false, karten_zu: Default::default(), doc: PixelDoc::open(root, rel, data)?, zoomansicht: Default::default(), raster: true, zwiebel: false, von: None, rechts: false, ueber: None, ansicht: Ansicht::Projekt, anim_t0: Some(std::time::Instant::now()) }) }
}

/// Ist die Datei ein Medium, das dieses Studio bearbeitet (Sprite, Player/Missile, Zeichensatz)? Liest nur die Art aus der Datei.
pub fn ist_pixel_medium(pfad: &std::path::Path) -> bool {
    let Ok(t) = std::fs::read_to_string(pfad) else { return false };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) else { return false };
    matches!(v["kind"].as_str(), Some("sprite" | "pmg" | "charset"))
}


/// Sichtbarer Name der Palette, die für das Ziel der Datei gilt (siehe `data/palettes/index.json`)
/// Videonorm der Projektdatei für das Ziel (`variant` im Eintrag des Ziels; sonst PAL)
pub fn projektnorm(pf: Option<&vc_ide::projfile::ProjectFile>, ziel: &str) -> &'static str {
    let v = pf.and_then(|pf| pf.targets.iter().find(|t| t.system == ziel || (t.system.is_empty() && t.id == ziel))).map(|t| t.variant.to_ascii_lowercase());
    if v.as_deref() == Some("ntsc") { "ntsc" } else { "pal" }
}
pub fn palettenname(ziel: &str) -> String { match ziel { "atari8" => vc_i18n::t!("studio-atari-gtia-pal"), "vic20" => "VIC-20".to_string(), "mega65" => "MEGA65".to_string(), "plus4" => vc_i18n::t!("studio-c64-farben-ted-palette-fehlt"), _ => "C64".to_string() } }

/// Gezeichnete Symbole der Werkzeugleiste (keine Schriftzeichen: keine Rückfallschrift)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sym { Stift, Radierer, Fuellen, Linie, Rahmen, Auswahl, Pipette, Undo, Redo }
struct SymZ { s: Sym, farbe: Color }
impl canvas::Program<Msg> for SymZ {
    type State = ();
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = Frame::new(r, b.size()); let k = b.width / 16.0; let p = |x: f32, y: f32| iced::Point::new(x * k, y * k);
        let strich = Stroke::default().with_color(self.farbe).with_width(1.6 * k).with_line_cap(canvas::LineCap::Round).with_line_join(canvas::LineJoin::Round);
        let fuell = |f: &mut Frame, pf: &Path| f.fill(pf, self.farbe);
        match self.s {
            Sym::Stift => { f.stroke(&Path::line(p(3.0, 13.0), p(11.0, 5.0)), Stroke { width: 3.0 * k, ..strich }); fuell(&mut f, &Path::new(|b| { b.move_to(p(10.0, 3.5)); b.line_to(p(12.5, 6.0)); b.line_to(p(14.0, 4.5)); b.line_to(p(11.5, 2.0)); b.close() })); fuell(&mut f, &Path::new(|b| { b.move_to(p(2.0, 14.0)); b.line_to(p(3.0, 11.5)); b.line_to(p(4.5, 13.0)); b.close() })) }
            Sym::Radierer => { f.stroke(&Path::new(|b| { b.move_to(p(3.0, 10.5)); b.line_to(p(8.5, 3.5)); b.line_to(p(13.0, 7.0)); b.line_to(p(8.5, 13.0)); b.line_to(p(5.5, 13.0)); b.close() }), strich); f.stroke(&Path::line(p(6.0, 7.0), p(10.5, 10.5)), strich) }
            Sym::Fuellen => { f.stroke(&Path::new(|b| { b.move_to(p(3.0, 8.0)); b.line_to(p(8.0, 3.0)); b.line_to(p(12.5, 7.5)); b.line_to(p(7.5, 12.5)); b.close() }), strich); fuell(&mut f, &Path::new(|b| { b.move_to(p(13.0, 10.0)); b.quadratic_curve_to(p(15.0, 13.0), p(13.0, 14.0)); b.quadratic_curve_to(p(11.0, 13.0), p(13.0, 10.0)); b.close() })) }
            Sym::Linie => { f.stroke(&Path::line(p(3.0, 13.0), p(13.0, 3.0)), strich); fuell(&mut f, &Path::circle(p(3.0, 13.0), 1.6 * k)); fuell(&mut f, &Path::circle(p(13.0, 3.0), 1.6 * k)) }
            Sym::Rahmen => f.stroke(&Path::rectangle(p(3.0, 4.0), iced::Size::new(10.0 * k, 8.0 * k)), strich),
            Sym::Auswahl => f.stroke(&Path::rectangle(p(3.0, 3.0), iced::Size::new(10.0 * k, 10.0 * k)), Stroke { line_dash: canvas::LineDash { segments: &[2.5 * k, 2.0 * k], offset: 0 }, ..strich }),
            Sym::Pipette => { f.stroke(&Path::line(p(4.0, 12.0), p(10.0, 6.0)), Stroke { width: 2.6 * k, ..strich }); f.stroke(&Path::line(p(9.0, 4.5), p(11.5, 7.0)), strich); fuell(&mut f, &Path::circle(p(12.5, 3.5), 2.2 * k)); fuell(&mut f, &Path::circle(p(3.0, 13.0), 1.2 * k)) }
            Sym::Undo => { f.stroke(&Path::new(|b| { b.move_to(p(5.0, 6.0)); b.line_to(p(9.0, 6.0)); b.quadratic_curve_to(p(14.0, 6.0), p(14.0, 10.0)); b.quadratic_curve_to(p(14.0, 13.0), p(10.0, 13.0)); b.line_to(p(7.0, 13.0)) }), strich); fuell(&mut f, &Path::new(|b| { b.move_to(p(2.0, 6.0)); b.line_to(p(6.0, 2.5)); b.line_to(p(6.0, 9.5)); b.close() })) }
            Sym::Redo => { f.stroke(&Path::new(|b| { b.move_to(p(11.0, 6.0)); b.line_to(p(7.0, 6.0)); b.quadratic_curve_to(p(2.0, 6.0), p(2.0, 10.0)); b.quadratic_curve_to(p(2.0, 13.0), p(6.0, 13.0)); b.line_to(p(9.0, 13.0)) }), strich); fuell(&mut f, &Path::new(|b| { b.move_to(p(14.0, 6.0)); b.line_to(p(10.0, 2.5)); b.line_to(p(10.0, 9.5)); b.close() })) }
        }
        vec![f.into_geometry()]
    }
}
pub fn sym<'a>(s: Sym, farbe: Color) -> Element<'a, Msg> { canvas::Canvas::new(SymZ { s, farbe }).width(16).height(16).into() }

/// Mausereignis der Zeichenfläche (Zelle in Pixeln des Mediums)
#[derive(Clone, Debug, PartialEq)]
pub enum Maus { Unten { x: usize, y: usize, rechts: bool }, Zug { x: usize, y: usize }, Los { x: usize, y: usize }, Ueber(Option<(usize, usize)>) }

/// Aktionen der Werkzeugleiste und der Seitenspalten
#[derive(Clone, Debug, PartialEq)]
pub enum Aktion { FarbSlot(usize), Ansicht(Ansicht), Werkzeug(Tool), Undo, Redo, Spiegeln(&'static str), Verschieben(&'static str), Leeren, ZoomPlus, ZoomMinus, ZoomAnpassen, Schieben(f32, f32), Zoom(f32), Mehr, Karte(&'static str), Raster, Zwiebel, Speichern, Exportieren, Ausschneiden, Kopieren, Einfuegen, AuswahlLoeschen, AuswahlAlles, AuswahlAus, PngLaden, Vorlage(String), FrameNeu, FrameDup, FrameLoeschen, Frame(usize), Wert(u8), Farbe(String, usize), Modus(bool), Flag(u8), Anim(AnimOp), AnimWahl(usize) }

/// Geometrie der Zeichenfläche: Zellbreite, -höhe, linke obere Ecke des Rasters (gemeinsame Logik in `vc_gui_ui::flaeche`)
fn geometrie(b: iced::Size, doc: &PixelDoc, a: &vc_gui_ui::flaeche::Ansicht) -> (f32, f32, f32, f32) {
    let g = a.geometrie(b, doc.aspect * doc.width as f32, doc.height as f32); (g.z * doc.aspect, g.z, g.ox, g.oy)
}
/// Einheiten der Fläche für Zoom und Einpassen: Breite (mit Seitenverhältnis der Zelle) und Höhe
fn einheiten(doc: &PixelDoc) -> (f32, f32) { (doc.aspect * doc.width as f32, doc.height as f32) }

fn rel_s(r: &str) -> String { r.to_string() }

pub struct Flaeche<'a> { pub s: &'a PixelStudio, pub farben: &'a Farben, pub norm: &'static str, pub rel: String, pub pal: Pal }
#[derive(Default)] pub struct FlaechenZustand { unten: bool, ueber: Option<(usize, usize)>, gesten: vc_gui_ui::flaeche::Gesten }

impl<'a> canvas::Program<Msg> for Flaeche<'a> {
    type State = FlaechenZustand;
    fn update(&self, st: &mut FlaechenZustand, e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        use iced::mouse::{Button, Event as M};
        let doc = &self.s.doc; let (cw, ch, ox, oy) = geometrie(b.size(), doc, &self.s.zoomansicht);
        let zelle = |p: iced::Point| -> Option<(usize, usize)> { let (x, y) = ((p.x - ox) / cw, (p.y - oy) / ch); (x >= 0.0 && y >= 0.0 && (x as usize) < doc.width && (y as usize) < doc.height).then(|| (x as usize, y as usize)) };
        let rel = self.rel.clone(); let send = |m: Maus| Some(canvas::Action::publish(Msg::StudioMaus(rel.clone(), m)).and_capture());
        // Leertaste + Ziehen verschiebt, Cmd/Strg + Mausrad zoomt; Tasten bleiben bei der Befehlstabelle (nichts wird verbraucht)
        if let Some(g) = st.gesten.ereignis(e, b, c) {
            use vc_gui_ui::flaeche::Geste;
            return match g { Geste::Erfasst => Some(canvas::Action::capture()), Geste::Schieben(dx, dy) => Some(canvas::Action::publish(Msg::StudioAkt(rel_s(&self.rel), Aktion::Schieben(dx, dy))).and_capture()), Geste::Zoom(plus) => Some(canvas::Action::publish(Msg::StudioAkt(rel_s(&self.rel), if plus { Aktion::ZoomPlus } else { Aktion::ZoomMinus })).and_capture()) }
        }
        match e {
            iced::Event::Mouse(M::ButtonPressed(Button::Right)) if self.s.doc.tool == Tool::Select => { let p = c.position()?; c.position_in(b)?; Some(canvas::Action::publish(Msg::KontextAuf(crate::kontext::Art::Flaeche, p, Some(self.rel.clone()))).and_capture()) }
            iced::Event::Mouse(M::ButtonPressed(k @ (Button::Left | Button::Right))) => { let p = c.position_in(b)?; let (x, y) = zelle(p)?; st.unten = true; send(Maus::Unten { x, y, rechts: *k == Button::Right }) }
            iced::Event::Mouse(M::CursorMoved { .. }) if !st.unten => { let z = c.position_in(b).and_then(|p| zelle(p)); if z != st.ueber { st.ueber = z; Some(canvas::Action::publish(Msg::StudioMaus(rel.clone(), Maus::Ueber(z)))) } else { None } }
            iced::Event::Mouse(M::CursorMoved { .. }) if st.unten => { let p = c.position_in(b).or_else(|| c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y)))?; let (x, y) = zelle(p)?; send(Maus::Zug { x, y }) }
            iced::Event::Mouse(M::ButtonReleased(Button::Left | Button::Right)) if st.unten => { st.unten = false; let p = c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y))?; let (x, y) = zelle(p).unwrap_or((0, 0)); send(Maus::Los { x, y }) }
            _ => None,
        }
    }
    fn draw(&self, st: &FlaechenZustand, r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let doc = &self.s.doc; let pal = self.pal; let mut f = Frame::new(r, b.size()); let (cw, ch, ox, oy) = geometrie(b.size(), doc, &self.s.zoomansicht);
        let (w, h) = (doc.width, doc.height);
        f.fill_rectangle(iced::Point::new(ox, oy), iced::Size::new(cw * w as f32, ch * h as f32), pal.editor);
        let rect = |x: usize, y: usize, c: Color, f: &mut Frame| f.fill_rectangle(iced::Point::new(ox + x as f32 * cw, oy + y as f32 * ch), iced::Size::new(cw, ch), c);
        for y in 0..h { for x in 0..w {
            match doc.farb_index(doc.pixel(x, y)) { Some(i) => rect(x, y, self.farben.farbe(&doc.target, self.norm, i), &mut f), None => if (x + y) % 2 == 0 { rect(x, y, Color { a: 0.5, ..pal.panel2 }, &mut f) } }
        } }
        if self.s.zwiebel && doc.fi > 0 && doc.ist_multicolor(doc.fi - 1) == doc.ist_multicolor(doc.fi) {
            for y in 0..h { for x in 0..w { if doc.pixel(x, y) != 0 { continue } let pv = doc.frames[doc.fi - 1].get(y).and_then(|r| r.get(x)).copied().unwrap_or(0); if pv != 0 { if let Some(i) = doc.farb_index_frame(doc.fi - 1, pv) { rect(x, y, Color { a: 0.28, ..self.farben.farbe(&doc.target, self.norm, i) }, &mut f) } } } }
        }
        if self.s.raster {
            let linie = Stroke::default().with_color(Color { a: 0.13, ..pal.text }).with_width(1.0);
            for x in 0..=w { let px = ox + x as f32 * cw; f.stroke(&Path::line(iced::Point::new(px, oy), iced::Point::new(px, oy + ch * h as f32)), linie); }
            for y in 0..=h { let py = oy + y as f32 * ch; f.stroke(&Path::line(iced::Point::new(ox, py), iced::Point::new(ox + cw * w as f32, py)), linie); }
        }
        if let Some((x0, y0, x1, y1)) = doc.sel {
            let p = Path::rectangle(iced::Point::new(ox + x0 as f32 * cw, oy + y0 as f32 * ch), iced::Size::new((x1 - x0 + 1) as f32 * cw, (y1 - y0 + 1) as f32 * ch));
            f.stroke(&p, Stroke { line_dash: canvas::LineDash { segments: &[5.0, 4.0], offset: 0 }, ..Stroke::default().with_color(pal.accent).with_width(2.0) });
        }
        // Aktive Zelle (Zeiger): Doppelrahmen (innen Akzent 2 px, außen Textfarbe gestrichelt) plus Marken an beiden Achsen; auch bei Hohem Kontrast sichtbar
        if let Some((x, y)) = st.ueber.filter(|(x, y)| *x < w && *y < h) {
            let (px, py) = (ox + x as f32 * cw, oy + y as f32 * ch);
            f.stroke_rectangle(iced::Point::new(px - 1.0, py - 1.0), iced::Size::new(cw + 2.0, ch + 2.0), Stroke::default().with_color(pal.accent).with_width(2.0));
            f.stroke_rectangle(iced::Point::new(px - 3.5, py - 3.5), iced::Size::new(cw + 7.0, ch + 7.0), Stroke { line_dash: canvas::LineDash { segments: &[4.0, 3.0], offset: 0 }, ..Stroke::default().with_color(pal.text).with_width(1.5) });
            f.fill_rectangle(iced::Point::new(px, oy - 7.0), iced::Size::new(cw, 4.0), pal.accent); f.fill_rectangle(iced::Point::new(ox - 7.0, py), iced::Size::new(4.0, ch), pal.accent);
        }
        vec![f.into_geometry()]
    }
    fn mouse_interaction(&self, _st: &FlaechenZustand, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> iced::mouse::Interaction { iced::mouse::Interaction::Crosshair }
}

/// Kleine Vorschau eines Frames (Frame-Liste, Zeichensatz-Raster)
pub struct Vorschau<'a> { pub doc: &'a PixelDoc, pub fi: usize, pub farben: &'a Farben, pub norm: &'static str, pub pal: Pal }
impl<'a> canvas::Program<Msg> for Vorschau<'a> {
    type State = ();
    fn draw(&self, _s: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let d = self.doc; let mut f = Frame::new(r, b.size()); let Some(fr) = d.frames.get(self.fi) else { return vec![f.into_geometry()] };
        let mc = d.ist_multicolor(self.fi); let w = fr.first().map_or(1, |z| z.len()).max(1); let h = fr.len().max(1); let asp = if d.kind == "sprite" && mc && d.frame_modes.len() > 1 { d.frame_modes[1].aspect } else { d.aspect };
        let z = ((b.width / (w as f32 * asp)).min(b.height / h as f32)).max(0.5); let (cw, ch) = (z * asp, z); let (ox, oy) = ((b.width - cw * w as f32) / 2.0, (b.height - ch * h as f32) / 2.0);
        f.fill_rectangle(iced::Point::new(ox, oy), iced::Size::new(cw * w as f32, ch * h as f32), self.pal.editor);
        for (y, z) in fr.iter().enumerate() { for (x, v) in z.iter().enumerate() {
            let idx = { let mut t = d.farb_index_frame(self.fi, *v); if d.kind == "pmg" || d.kind == "sprite" { if *v == 0 { t = None } } t };
            if let Some(i) = idx { f.fill_rectangle(iced::Point::new(ox + x as f32 * cw, oy + y as f32 * ch), iced::Size::new(cw.ceil(), ch.ceil()), self.farben.farbe(&d.target, self.norm, i)); }
        } }
        vec![f.into_geometry()]
    }
}

impl App {
    /// Videonorm der Palette, die für das Ziel angezeigt wird (Wahl im Studio oder Variante des Projekts)
    pub(crate) fn norm_fuer(&self, ziel: &str, a: Ansicht) -> &'static str { match a { Ansicht::Pal => "pal", Ansicht::Ntsc => "ntsc", Ansicht::Projekt => projektnorm(self.proj.pf.as_ref(), ziel) } }
    /// Beschriftung der Palette über dem Farbwähler: Name, Videonorm und woher sie kommt
    pub(crate) fn palettenzeile(&self, ziel: &str, a: Ansicht) -> String {
        if !self.farben.hat_varianten(ziel) { return vc_i18n::t!("studio-palette-p1", p1 = palettenname(ziel)) }
        let name = match ziel { "atari8" => vc_i18n::t!("studio-atari-gtia"), _ => palettenname(ziel) }; let (norm, proj) = (self.norm_fuer(ziel, a), projektnorm(self.proj.pf.as_ref(), ziel));
        if norm == proj { vc_i18n::t!("studio-palette-name-p2-aus-dem-projekt", name = &name, p2 = norm.to_uppercase()) } else { vc_i18n::t!("studio-palette-name-p2-ansicht-weicht-vom", name = &name, p2 = norm.to_uppercase(), p3 = proj.to_uppercase()) }
    }
    fn ansicht_wahl(&self, pal: Pal, rel: &str, aktuell: Ansicht) -> Element<'_, Msg> { let r = rel.to_string(); self.ansicht_wahl_mit(pal, aktuell, move |a| Msg::StudioAkt(r.clone(), Aktion::Ansicht(a))) }
    pub(crate) fn ansicht_wahl_mit(&self, pal: Pal, aktuell: Ansicht, mach: impl Fn(Ansicht) -> Msg + 'static + Clone) -> Element<'_, Msg> {
        let seg = |name: &str, a: Ansicht, tipp: &str| -> Element<'static, Msg> {
            let on = aktuell == a; let b = button(text(name.to_string()).size(11).color(pal.text)).padding([2, 8]).on_press(mach(a)).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() });
            iced::widget::tooltip(b, container(text(tipp.to_string()).size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Bottom).gap(4).into()
        };
        iced::widget::row![seg("PAL", Ansicht::Pal, &vc_i18n::t!("studio-palette-fuer-pal-zeigen-nur-ansicht")), seg("NTSC", Ansicht::Ntsc, &vc_i18n::t!("studio-palette-fuer-ntsc-zeigen-nur-ansicht")), seg(&vc_i18n::t!("studio-projekt"), Ansicht::Projekt, &vc_i18n::t!("studio-videonorm-des-projekts-verwenden"))].spacing(4).into()
    }
    /// Öffnet (oder wählt) das Pixel-Studio zu einer Mediendatei
    pub(crate) fn studio_oeffnen(&mut self, rel: &str) {
        if !self.studios.contains_key(rel) {
            match PixelStudio::oeffnen(&self.root, rel, &self.data) { Ok(s) => { self.studios.insert(rel.to_string(), s); } Err(e) => { self.log.push(vc_i18n::t!("studio-rel-e", rel = &rel, e = &e)); return } }
        }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }
    /// Mausereignis der Zeichenfläche auf das Modell anwenden
    pub(crate) fn studio_maus(&mut self, rel: &str, m: Maus) {
        let Some(s) = self.studios.get_mut(rel) else { return }; let d = &mut s.doc;
        match m {
            Maus::Unten { x, y, rechts } => {
                s.rechts = rechts; let wert = if rechts || d.tool == Tool::Erase { 0 } else { d.pv };
                match d.tool {
                    Tool::Pick if !rechts => { d.pv = d.pixel(x, y); }
                    Tool::Pick => { d.push_undo(); d.setze(x, y, 0); }
                    Tool::Fill => { d.push_undo(); d.fuelle(x, y, wert); }
                    Tool::Line | Tool::Rect | Tool::Select => { s.von = Some((x, y)); if d.tool == Tool::Select { d.sel = Some((x, y, x, y)); } }
                    _ => { d.push_undo(); d.setze(x, y, wert); }
                }
                if rechts && matches!(d.tool, Tool::Line | Tool::Rect) { s.von = Some((x, y)) }
            }
            Maus::Ueber(z) => s.ueber = z,
            Maus::Zug { x, y } => {
                let wert = if d.tool == Tool::Erase || s.rechts { 0 } else { d.pv };
                match d.tool { Tool::Pen | Tool::Erase | Tool::Fill | Tool::Pick if s.rechts || matches!(d.tool, Tool::Pen | Tool::Erase) => { if d.pixel(x, y) != wert { d.setze(x, y, wert) } } Tool::Select => { if let Some(a) = s.von { d.sel = Some((a.0.min(x), a.1.min(y), a.0.max(x), a.1.max(y))) } } _ => {} }
            }
            Maus::Los { x, y } => {
                let a = s.von.take(); let wert = if s.rechts { 0 } else { d.pv }; s.rechts = false;
                if let Some(a) = a { match d.tool { Tool::Rect => { d.push_undo(); d.rahmen(a, (x, y), wert) } Tool::Line => { d.push_undo(); d.linie(a, (x, y), wert) } _ => {} } }
            }
        }
    }
    /// Aktion der Werkzeugleiste und Seitenspalten
    pub(crate) fn studio_aktion(&mut self, rel: &str, a: Aktion) {
        let data = self.data.clone();
        let Some(s) = self.studios.get_mut(rel) else { return }; let d = &mut s.doc;
        match a {
            Aktion::Werkzeug(t) => d.tool = t,
            Aktion::Undo => d.undo(), Aktion::Redo => d.redo(),
            Aktion::Spiegeln(w) => d.operation(w), Aktion::Verschieben(w) => d.operation(w), Aktion::Leeren => d.operation("clear"),
            Aktion::ZoomPlus | Aktion::ZoomMinus => { let (w, h) = einheiten(d); s.zoomansicht.stufe(matches!(a, Aktion::ZoomPlus), w, h) }
            Aktion::Schieben(dx, dy) => { let (w, h) = einheiten(d); s.zoomansicht.schieben(dx, dy, w, h) }
            Aktion::Mehr => s.mehr = !s.mehr,
            Aktion::FarbSlot(i) => s.farb_slot = i,
            Aktion::Karte(k) => { if !s.karten_zu.remove(k) { s.karten_zu.insert(k); } }
            Aktion::Zoom(z) => s.zoomansicht.setze(z),
            Aktion::ZoomAnpassen => s.zoomansicht.einpassen(),
            Aktion::Ansicht(a) => s.ansicht = a, Aktion::Raster => s.raster = !s.raster, Aktion::Zwiebel => s.zwiebel = !s.zwiebel,
            Aktion::Exportieren => { self.export_oeffnen() }
            Aktion::Kopieren | Aktion::Ausschneiden => { let schnitt = matches!(a, Aktion::Ausschneiden); if let Some((x0, y0, x1, y1)) = d.sel { d.auswahl_kopieren(schnitt); d.status = vc_i18n::t!("studio-p1-p2-p3-pixel", p1 = if schnitt { vc_i18n::t!("studio-ausgeschnitten") } else { vc_i18n::t!("studio-kopiert") }, p2 = x1 - x0 + 1, p3 = y1 - y0 + 1) } else { d.status = vc_i18n::t!("studio-nichts-ausgewaehlt-erst-mit-dem-auswahlwerkzeug").into() } }
            Aktion::Einfuegen => { if let Some(c) = d.clip.as_ref().map(|c| (c.first().map_or(0, |r| r.len()), c.len())) { let (x, y) = d.sel.map(|s| (s.0, s.1)).or(s.ueber).unwrap_or((0, 0)); if d.sel.is_none() { d.sel = Some((x, y, x, y)) } d.auswahl_einfuegen(); d.status = vc_i18n::t!("studio-eingefuegt-p1-p2-pixel-bei-x", p1 = c.0, p2 = c.1, x = &x, y = &y) } else { d.status = vc_gui_ui::befehle::plattform_text(&vc_i18n::t!("studio-zwischenablage-leer-erst-kopieren-cmd-c")) } } Aktion::AuswahlLoeschen => d.auswahl_loeschen(),
            Aktion::AuswahlAlles => d.sel = Some((0, 0, d.width.saturating_sub(1), d.height.saturating_sub(1))), Aktion::AuswahlAus => d.sel = None,
            Aktion::PngLaden => {}
            Aktion::Vorlage(r) => { if let Err(e) = d.vorlage_anwenden_mit(&data, &r) { d.status = e.clone(); self.log.push(e) } }
            Aktion::Speichern => match d.save() { Ok(()) => self.log.push(vc_i18n::t!("studio-gespeichert-rel", rel = &rel)), Err(e) => self.log.push(e) },
            Aktion::FrameNeu => d.frame_anhaengen(), Aktion::FrameDup => d.frame_duplizieren(), Aktion::FrameLoeschen => d.frame_loeschen(),
            Aktion::Frame(i) => d.frame_waehlen(i),
            Aktion::Wert(v) => d.pv = v,
            Aktion::Farbe(k, i) => d.farbe_setzen(&k, i),
            Aktion::Modus(mc) => d.modus_wechseln(mc),
            Aktion::Flag(bit) => { let mut bits = vc_ide_model::pixel::flags_from_hw(&d.media["hw"]); bits ^= bit; d.push_undo(); vc_ide_model::pixel::flags_to_hw(&mut d.media, bits) }
            Aktion::Anim(op) => { if let Err(e) = d.anim_aendern(op) { self.log.push(e) } }
            Aktion::AnimWahl(i) => { d.sel_anim = i; s.anim_t0 = Some(std::time::Instant::now()) }
        }
    }
    /// PNG-Datei in das gewählte Frame übernehmen
    pub(crate) fn png_laden(&mut self, rel: &str, p: &std::path::Path) {
        let (root, data) = (self.root.clone(), self.data.clone());
        let Some(s) = self.studios.get_mut(rel) else { return };
        let r = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display())).and_then(|b| s.doc.png_importieren(&root, &data, &b));
        match r { Ok(m) => self.log.push(m), Err(e) => { s.doc.status = e.clone(); self.log.push(e) } }
    }
    /// Name und Beschreibung der Zeichenfläche für Bildschirmleser (aktives Studio)
    pub(crate) fn studio_a11y(&self) -> Option<(String, String)> {
        if let Some(x) = self.disk_a11y() { return Some(x) } if let Some(x) = self.screen_a11y() { return Some(x) } if let Some(x) = self.tile_a11y() { return Some(x) } if let Some(x) = self.bank_a11y() { return Some(x) } if let Some(x) = self.song_a11y() { return Some(x) }
        let n = self.name()?; let s = self.studios.get(&n)?; let d = &s.doc;
        let wert = d.values.get(d.pv as usize).cloned().unwrap_or_else(|| vc_i18n::t!("studio-wert-p1", p1 = d.pv + 1));
        Some((vc_i18n::t!("studio-zeichenflaeche-n-p2", n = &n, p2 = d.titel()), vc_i18n::t!("studio-beschreibung", fi = d.fi, n = d.frames.len(), art = if d.kind == "sprite" { if d.ist_multicolor(d.fi) { format!(", {}", vc_i18n::t!("studio-multicolor")) } else { format!(", {}", vc_i18n::t!("studio-hires")) } } else { String::new() }, w = d.width, h = d.height, werkzeug = d.tool.label(), wert = &wert, stand = if d.dirty() { vc_i18n::t!("studio-ungespeichert") } else { vc_i18n::t!("studio-gespeichert") })))
    }
    /// Einzeltasten im Studio (nur ohne Cmd/Strg/Alt, wenn kein Textfeld den Fokus hat): D Stift, E Radierer, F Füllen, L Linie, R Rahmen, S Auswahl, I Pipette, X/Y Spiegeln, 1–4 Wert, ←/→ Frame
    pub(crate) fn studio_taste(&mut self, rel: &str, k: &keyboard::Key, m: keyboard::Modifiers) -> bool {
        if m.command() && !m.alt() && !m.shift() && matches!(k, keyboard::Key::Character(c) if c.as_str().eq_ignore_ascii_case("a")) { self.studio_aktion(rel, Aktion::AuswahlAlles); return true }
        if m.command() && !m.alt() && !m.shift() { if let keyboard::Key::Character(c) = k { let a = match c.as_str().to_lowercase().as_str() { "c" => Some(Aktion::Kopieren), "x" => Some(Aktion::Ausschneiden), "v" => Some(Aktion::Einfuegen), _ => None }; if let Some(a) = a { self.studio_aktion(rel, a); return true } } }
        if m.command() || m.alt() || m.control() { return false }
        let a = match k {
            keyboard::Key::Character(c) => match c.as_str().to_lowercase().as_str() {
                "d" => Aktion::Werkzeug(Tool::Pen), "e" => Aktion::Werkzeug(Tool::Erase), "f" => Aktion::Werkzeug(Tool::Fill), "l" => Aktion::Werkzeug(Tool::Line), "r" => Aktion::Werkzeug(Tool::Rect),
                "s" => Aktion::Werkzeug(Tool::Select), "i" => Aktion::Werkzeug(Tool::Pick), "x" => Aktion::Spiegeln("mh"), "y" => Aktion::Spiegeln("mv"),
                n @ ("1" | "2" | "3" | "4") => Aktion::Wert(n.parse::<u8>().unwrap_or(1) - 1),
                "+" | "=" => Aktion::ZoomPlus, "-" => Aktion::ZoomMinus, "0" => Aktion::ZoomAnpassen,
                // C: Hires ↔ Multicolor des Frames (nur Sprites, die beide Modi kennen)
                "c" => match self.studios.get(rel) { Some(s) if s.doc.kind == "sprite" && s.doc.frame_modes.len() > 1 => Aktion::Modus(!s.doc.ist_multicolor(s.doc.fi)), _ => return false },
                _ => return false },
            keyboard::Key::Named(keyboard::key::Named::Delete | keyboard::key::Named::Backspace) => Aktion::AuswahlLoeschen,
            keyboard::Key::Named(keyboard::key::Named::Escape) => Aktion::AuswahlAus,
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => { let i = self.studios.get(rel).map_or(0, |s| s.doc.fi); Aktion::Frame(i.saturating_sub(1)) }
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) => { let i = self.studios.get(rel).map_or(0, |s| s.doc.fi); Aktion::Frame(i + 1) }
            _ => return false,
        };
        self.studio_aktion(rel, a); true
    }

    /// Ansicht des Pixel-Studios: Werkzeugleiste, Frame-Liste, Zeichenfläche, Seitenspalte
    pub(crate) fn studio_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        // Die Breitenklasse folgt der tatsächlichen Breite des Studios (auch im geteilten Dokumentenfeld), nicht der des Fensters
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.studio_ansicht_k(pal, &rel, muster::klasse(size.width), size.width)).into()
    }
    fn studio_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse, breite: f32) -> Element<'_, Msg> {
        // sehr schmal (z. B. halbes Dokumentenfeld): nur die ersten drei Werkzeuge in der Leiste, die übrigen im ⋯-Menü
        let sehr_schmal = klasse == muster::Klasse::Schmal && breite < 460.0;
        let Some(s) = self.studios.get(rel) else { return text("").into() };
        let d = &s.doc; let r = rel.to_string(); let norm = self.norm_fuer(&d.target, s.ansicht);
        let knopf = |name: String, tipp: String, an: bool, a: Aktion, aktiv: bool| -> Element<'static, Msg> { muster::knopf(pal, &name, &tipp, an.then(|| Msg::StudioAkt(r.clone(), a)), aktiv, false) };
        let knopf_s = |sy: Sym, tipp: String, an: bool, a: Aktion, aktiv: bool| -> Element<'static, Msg> { muster::knopf_el(pal, sym(sy, if an { pal.text } else { pal.mute }), &tipp, an.then(|| Msg::StudioAkt(r.clone(), a)), aktiv, false, true) };
        let wk = |t: Tool, sy: Sym, tipp: &str| knopf_s(sy, tipp.to_string(), true, Aktion::Werkzeug(t), d.tool == t);
        // Zone 2: genau eine Leistenreihe. Werkzeuge · Bearbeiten · (Transformieren) · Ansicht; was nicht passt, steht im ⋯-Menü (Karte über der Fläche)
        let werkzeuge_a = || iced::widget::row![wk(Tool::Pen, Sym::Stift, &vc_i18n::t!("studio-stift-d-rechte-taste-loescht")), wk(Tool::Erase, Sym::Radierer, &vc_i18n::t!("studio-radierer-e")), wk(Tool::Fill, Sym::Fuellen, &vc_i18n::t!("studio-fuellen-f"))].spacing(4).align_y(Alignment::Center);
        let werkzeuge_b = || iced::widget::row![wk(Tool::Line, Sym::Linie, &vc_i18n::t!("studio-linie-l")), wk(Tool::Rect, Sym::Rahmen, &vc_i18n::t!("studio-rahmen-r")), wk(Tool::Select, Sym::Auswahl, &vc_i18n::t!("studio-auswahl-s-rechtsklick-menue")), wk(Tool::Pick, Sym::Pipette, &vc_i18n::t!("studio-pipette-i"))].spacing(4).align_y(Alignment::Center);
        let tools: Element<Msg> = if sehr_schmal { werkzeuge_a().into() } else { iced::widget::row![werkzeuge_a(), werkzeuge_b()].spacing(4).into() };
        let bearbeiten = iced::widget::row![knopf_s(Sym::Undo, vc_i18n::t!("studio-rueckgaengig-cmd-z").into(), d.kann_undo(), Aktion::Undo, false), knopf_s(Sym::Redo, vc_i18n::t!("studio-wiederholen-cmd-umschalt-z").into(), d.kann_redo(), Aktion::Redo, false)].spacing(4);
        let transformieren = iced::widget::row![
            knopf("⇋".into(), vc_i18n::t!("studio-horizontal-spiegeln-x").into(), true, Aktion::Spiegeln("mh"), false), knopf("⇅".into(), vc_i18n::t!("studio-vertikal-spiegeln-y").into(), true, Aktion::Spiegeln("mv"), false),
            knopf("←".into(), vc_i18n::t!("studio-nach-links-verschieben").into(), true, Aktion::Verschieben("sl"), false), knopf("→".into(), vc_i18n::t!("studio-nach-rechts-verschieben").into(), true, Aktion::Verschieben("sr"), false), knopf("↑".into(), vc_i18n::t!("studio-nach-oben-verschieben").into(), true, Aktion::Verschieben("su"), false), knopf("↓".into(), vc_i18n::t!("studio-nach-unten-verschieben").into(), true, Aktion::Verschieben("sd"), false),
            knopf(vc_i18n::t!("studio-leeren").into(), vc_i18n::t!("studio-frame-leeren").into(), true, Aktion::Leeren, false)].spacing(4).align_y(Alignment::Center);
        let (zew, zeh) = einheiten(d); let zoom_wert = format!("{}×", s.zoomansicht.zoom_wert(zew, zeh));
        let zoomgruppe = iced::widget::row![knopf("−".into(), vc_i18n::t!("studio-verkleinern-cmd-mausrad").into(), true, Aktion::ZoomMinus, false), text(zoom_wert).size(muster::schrift::TEXT).font(vc_gui_ui::fonts::MONO).color(pal.text).width(40).align_x(iced::alignment::Horizontal::Center), knopf("+".into(), vc_i18n::t!("studio-vergroessern-cmd-mausrad").into(), true, Aktion::ZoomPlus, false), knopf(vc_i18n::t!("studio-einpassen").into(), vc_i18n::t!("studio-flaeche-einpassen-0").into(), true, Aktion::ZoomAnpassen, s.zoomansicht.zoom.is_none())].spacing(4).align_y(Alignment::Center);
        let ansicht = iced::widget::row![knopf(if s.raster { vc_i18n::t!("studio-raster").into() } else { vc_i18n::t!("studio-raster-2").into() }, vc_i18n::t!("studio-raster-ein-aus").into(), true, Aktion::Raster, s.raster), knopf(if s.zwiebel { vc_i18n::t!("studio-onion-skin").into() } else { vc_i18n::t!("studio-onion-skin-2").into() }, vc_i18n::t!("studio-vorheriges-frame-durchscheinen-lassen").into(), d.fi > 0, Aktion::Zwiebel, s.zwiebel)].spacing(4);
        let transf_knopf = knopf(vc_i18n::t!("studio-transformieren").into(), vc_i18n::t!("studio-spiegeln-verschieben-leeren-alle-aktionen-menue").into(), true, Aktion::Mehr, s.mehr);
        let mehr_knopf = knopf("⋯".into(), vc_i18n::t!("studio-weitere-werkzeuge-und-ansicht-alle-aktionen").into(), true, Aktion::Mehr, s.mehr);
        let leiste_inhalt: Element<Msg> = match klasse {
            muster::Klasse::Breit => iced::widget::row![tools, muster::trenner(pal), bearbeiten, muster::trenner(pal), transf_knopf, space::horizontal(), zoomgruppe, ansicht].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Mittel => iced::widget::row![tools, muster::trenner(pal), bearbeiten, space::horizontal(), zoomgruppe, mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Schmal => iced::widget::row![tools, muster::trenner(pal), bearbeiten, space::horizontal(), mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
        };
        let leiste = muster::leiste(pal, leiste_inhalt);
        // Frames
        let mut frames = iced::widget::column![].spacing(4);
        if d.kind == "charset" {
            frames = frames.push(text(vc_i18n::t!("studio-zeichen-p1-p2", p1 = &d.fi, p2 = format!("{:02X}", d.fi))).size(12).color(pal.text));
            for zeile in 0..(d.frames.len() + 15) / 16 {
                let mut z = iced::widget::row![].spacing(2);
                for i in zeile * 16..((zeile + 1) * 16).min(d.frames.len()) {
                    z = z.push(button(canvas::Canvas::new(Vorschau { doc: d, fi: i, farben: &self.farben, norm, pal }).width(14).height(14)).padding(1).on_press(Msg::StudioAkt(r.clone(), Aktion::Frame(i))).style(move |_, _| button::Style { background: None, border: Border { color: if i == d.fi { pal.accent } else { Color::TRANSPARENT }, width: 2.0, radius: 2.0.into() }, ..Default::default() }));
                }
                frames = frames.push(z);
            }
        } else {
            frames = frames.push(iced::widget::row![text(vc_i18n::t!("studio-frames-p1", p1 = d.frames.len())).size(muster::schrift::HINWEIS).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute), space::horizontal(), if d.fixed_frames { Element::from(space::horizontal().width(0)) } else { knopf("+".into(), vc_i18n::t!("studio-leeres-frame-anhaengen").into(), true, Aktion::FrameNeu, false) }].align_y(Alignment::Center));
            for i in 0..d.frames.len() {
                let name = d.frame_names.get(i).cloned().unwrap_or_else(|| vc_i18n::t!("studio-frame-i", i = &i));
                let wahl = i == d.fi;
                frames = frames.push(button(iced::widget::column![canvas::Canvas::new(Vorschau { doc: d, fi: i, farben: &self.farben, norm, pal }).width(Length::Fill).height(46), text(name).size(11).color(pal.mute)].spacing(2)).padding(3).width(Length::Fill).on_press(Msg::StudioAkt(r.clone(), Aktion::Frame(i))).style(move |_, _| button::Style { background: Some(pal.panel.into()), border: Border { color: if wahl { pal.accent } else { pal.line }, width: if wahl { 2.0 } else { 1.0 }, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }));
            }
        }
        if d.kind != "charset" && !d.fixed_frames { frames = frames.push(iced::widget::row![knopf(vc_i18n::t!("studio-dupl").into(), vc_i18n::t!("studio-frame-duplizieren").into(), true, Aktion::FrameDup, false), knopf(vc_i18n::t!("studio-loeschen").into(), vc_i18n::t!("studio-frame-loeschen").into(), d.frames.len() > 1, Aktion::FrameLoeschen, false)].spacing(4).wrap().vertical_spacing(4)) }
        // Seitenspalte: Werte, Farben, Modus, Flags
        let mut werkzeug = iced::widget::column![].spacing(5); let mut dokument = iced::widget::column![].spacing(5); let mut info = iced::widget::column![].spacing(5);
        let nwerte = 1usize << d.bpp.min(2);
        { let mut zeile = iced::widget::row![].spacing(muster::abstand::ENG);
          let n = nwerte.max(2).min(4);
          for v in 0..n {
            let name = d.values.get(v).cloned().unwrap_or_else(|| vc_i18n::t!("studio-wert-v", v = &v)); let wahl = d.pv as usize == v;
            let kurz = name.split(" (").next().unwrap_or(&name).to_string();
            let farbe = d.farb_index(v as u8).map(|i| self.farben.farbe(&d.target, norm, i));
            let chip = button(iced::widget::row![container(space::vertical()).width(16).height(16).style(move |_| container::Style { background: Some(farbe.unwrap_or(pal.panel2).into()), border: Border { color: pal.line, width: 1.0, radius: 3.0.into() }, ..Default::default() }), text(vc_i18n::t!("studio-p1-kurz", p1 = v + 1, kurz = &kurz)).size(muster::schrift::HINWEIS + 1.0).color(pal.text)].spacing(6).align_y(Alignment::Center)).width(Length::FillPortion(1)).padding([4, 6]).on_press(Msg::StudioAkt(r.clone(), Aktion::Wert(v as u8)))
                .style(move |_, _| button::Style { background: Some(if wahl { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if wahl { pal.accent } else { pal.line }, width: if wahl { 2.0 } else { 1.0 }, radius: muster::RADIUS_KNOPF.into() }, text_color: pal.text, ..Default::default() });
            zeile = zeile.push(iced::widget::tooltip(chip, container(text(name.clone()).size(muster::schrift::HINWEIS).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), tooltip::Position::Bottom).gap(4));
            if (v + 1) % 2 == 0 || v + 1 == n { werkzeug = werkzeug.push(zeile); zeile = iced::widget::row![].spacing(muster::abstand::ENG); }
          } }
        if d.kind == "sprite" && d.frame_modes.len() > 1 {
            let mc = d.ist_multicolor(d.fi);
            dokument = dokument.push(text(vc_i18n::t!("studio-modus-dieses-frames")).size(11).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute)).push(iced::widget::row![knopf(vc_i18n::t!("studio-hires").into(), vc_i18n::t!("studio-eine-farbe-24-pixel-breit-c").into(), true, Aktion::Modus(false), !mc), knopf(vc_i18n::t!("studio-multicolor").into(), vc_i18n::t!("studio-drei-farben-12-breite-pixel-c").into(), true, Aktion::Modus(true), mc)].spacing(4));
        }
        {
            let slots = d.farben(); let wahl_i = s.farb_slot.min(slots.len().saturating_sub(1));
            let mut felder: Vec<Element<Msg>> = vec![];
            for (i, (key, label)) in slots.iter().enumerate() {
                let idx = d.farbe(key, 1); let c = self.farben.farbe(&d.target, norm, idx); let an = i == wahl_i;
                let feld = button(container(space::vertical()).width(Length::Fill).height(34).style(move |_| container::Style { background: Some(c.into()), border: Border { color: pal.line, width: 1.0, radius: 4.0.into() }, ..Default::default() })).padding(3).width(Length::FillPortion(1)).on_press(Msg::StudioAkt(r.clone(), Aktion::FarbSlot(i)))
                    .style(move |_, _| button::Style { background: Some(pal.panel.into()), border: Border { color: if an { pal.accent } else { pal.line }, width: if an { 2.0 } else { 1.0 }, radius: muster::RADIUS_KNOPF.into() }, ..Default::default() });
                felder.push(iced::widget::column![feld, text(label.clone()).size(muster::schrift::HINWEIS).color(if an { pal.text } else { pal.mute }), text(vc_i18n::t!("studio-index-idx", idx = &idx)).size(muster::schrift::HINWEIS - 1.0).color(pal.mute)].spacing(2).width(Length::FillPortion(1)).into());
            }
            // bis zu drei Farbfelder nebeneinander, bei vier (Zeichensatz: Zeichenfarbe, Hintergrund, zwei Multicolor) zwei mal zwei, damit die Beschriftungen nicht ineinanderlaufen
            let je = if felder.len() > 3 { 2 } else { felder.len().max(1) }; let mut zeilen = iced::widget::column![].spacing(muster::abstand::GRUPPE); let mut it = felder.into_iter();
            loop { let mut z = iced::widget::row![].spacing(muster::abstand::GRUPPE); let mut n = 0; for _ in 0..je { if let Some(f) = it.next() { z = z.push(f); n += 1 } } if n == 0 { break } for _ in n..je { z = z.push(space::horizontal().width(Length::FillPortion(1))) } zeilen = zeilen.push(z); if n < je { break } }
            dokument = dokument.push(zeilen);
            if let Some((key, label)) = slots.get(wahl_i) {
                let aktuell = d.farbe(key, 1);
                dokument = dokument.push(text(vc_i18n::t!("studio-label-waehlen", label = &label)).size(muster::schrift::HINWEIS).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute));
                dokument = dokument.push(text(self.palettenzeile(&d.target, s.ansicht)).size(muster::schrift::HINWEIS).color(pal.mute));
                if self.farben.hat_varianten(&d.target) { dokument = dokument.push(self.ansicht_wahl(pal, &r, s.ansicht)) }
                let mut raster = iced::widget::column![].spacing(2); let mut zeile = iced::widget::row![].spacing(2);
                let eintr = self.farben.eintraege(&d.target, norm); let breite = 8; let kante = if eintr.len() > 32 { 14 } else { 24 };
                for (k, (idx, c)) in eintr.iter().enumerate() {
                    let (idx, c, key2, gewaehlt) = (*idx as usize, *c, key.clone(), *idx as usize == aktuell);
                    zeile = zeile.push(button(space::vertical()).width(kante).height(kante).padding(0).on_press(Msg::StudioAkt(r.clone(), Aktion::Farbe(key2, idx))).style(move |_, _| button::Style { background: Some(c.into()), border: Border { color: if gewaehlt { pal.accent } else { pal.line }, width: if gewaehlt { 2.0 } else { 1.0 }, radius: 3.0.into() }, ..Default::default() }));
                    if (k + 1) % breite == 0 { raster = raster.push(zeile); zeile = iced::widget::row![].spacing(2); }
                }
                dokument = dokument.push(raster.push(zeile));
            }
        }
        if d.kind == "charset" {
            let vorlagen = d.vorlagen(&self.data);
            if !vorlagen.is_empty() {
                dokument = dokument.push(text(vc_i18n::t!("studio-vorlage")).size(11).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute));
                for (ref_, label, fehlt) in vorlagen { let aktuell = d.vorlage_aktuell() == ref_; dokument = dokument.push(knopf(vc_i18n::t!("studio-label-p2", label = &label, p2 = if fehlt { format!("  {}", vc_i18n::t!("studio-rom-fehlt")) } else { String::new() }), vc_i18n::t!("studio-zeichensatz-vorlage-laden-rueckgaengig-stellt-wieder").into(), !fehlt, Aktion::Vorlage(ref_), aktuell)) }
            }
        }
        // Herkunftskarte (Zeichensätze): Etikett immer als Wort (EIGEN, EIGEN, GEÄNDERT, AUS DEINEM ROM, ROM FEHLT), nie nur Farbe
        let herkunft = if d.kind == "charset" { d.herkunft(&self.data) } else { None };
        if let Some((etikett, zeilen)) = &herkunft {
            info = info.push(container(text(etikett.to_string()).size(muster::schrift::HINWEIS).font(vc_gui_ui::fonts::UI_FETT).color(pal.accent)).padding([2, 8]).style(move |_| container::Style { border: Border { color: pal.accent, width: if pal.hoch { 2.0 } else { 1.0 }, radius: muster::RADIUS_KNOPF.into() }, ..Default::default() }));
            for z in zeilen { info = info.push(text(z.clone()).size(muster::schrift::HINWEIS).color(pal.mute).width(Length::Fill)) }
        }
        if vc_ide_model::pixel::has_sprite_flags(&d.target, &d.kind) {
            let bits = vc_ide_model::pixel::flags_from_hw(&d.media["hw"]);
            let flag = |bit: u8, label: &str, tipp: &str| -> Element<'static, Msg> { let r2 = r.clone(); iced::widget::tooltip(iced::widget::checkbox(bits & bit != 0).label(label.to_string()).size(16).text_size(muster::schrift::TEXT).on_toggle(move |_| Msg::StudioAkt(r2.clone(), Aktion::Flag(bit))), container(text(tipp.to_string()).size(muster::schrift::HINWEIS).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), tooltip::Position::Right).gap(4).into() };
            info = info.push(iced::widget::column![flag(2, &vc_i18n::t!("studio-doppelt-breit"), &vc_i18n::t!("studio-x2-doppelte-breite-d01d")), flag(4, &vc_i18n::t!("studio-doppelt-hoch"), &vc_i18n::t!("studio-y2-doppelte-hoehe-d017")), flag(8, &vc_i18n::t!("studio-hinter-dem-hintergrund"), &vc_i18n::t!("studio-bg-hinter-dem-hintergrund-d01b"))].spacing(muster::abstand::GRUPPE));
        }
        // Animationen (Sprites): Folgen aus Frames mit Dauer in Bildern (1/50 s), Vorschau läuft in Echtzeit
        if d.kind == "sprite" {
            info = info.push(text(vc_i18n::t!("studio-animationen")).size(11).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute));
            match d.animationen() {
                Err(e) => info = info.push(text(vc_i18n::t!("studio-fehlerhaft-e", e = &e)).size(11).color(pal.err).width(Length::Fill)),
                Ok(list) => {
                    info = info.push(iced::widget::row![knopf(vc_i18n::t!("studio-neu").into(), vc_i18n::t!("studio-neue-animation-mit-dem-gewaehlten-frame").into(), true, Aktion::Anim(AnimOp::New), false)].spacing(4).wrap().vertical_spacing(4));
                    if list.is_empty() { info = info.push(text(vc_i18n::t!("studio-noch-keine-animation-neu-legt-eine")).size(11).color(pal.mute).width(Length::Fill)) }
                    let n = d.sel_anim.min(list.len().saturating_sub(1));
                    let mut reihe = iced::widget::row![].spacing(3);
                    for (i, a) in list.iter().enumerate() { reihe = reihe.push(knopf(a.name.clone(), vc_i18n::t!("studio-animation-waehlen").into(), true, Aktion::AnimWahl(i), i == n)) }
                    info = info.push(reihe.wrap());
                    if let Some(a) = list.get(n) {
                        // Vorschau: das Frame, das jetzt an der Reihe ist
                        let t = s.anim_t0.map_or(0.0, |t| t.elapsed().as_secs_f64()); let ex = a.expanded(); let schritt = a.step_at(t); let fi = ex.get(schritt).map_or(0, |x| x.0);
                        info = info.push(iced::widget::row![canvas::Canvas::new(Vorschau { doc: d, fi, farben: &self.farben, norm, pal }).width(60).height(52), text(vc_i18n::t!("studio-schritt-p1-p2-p3-bilder-p4", p1 = schritt + 1, p2 = ex.len().max(1), p3 = a.total_ticks(), p4 = vc_i18n::num((a.total_ticks() as f32 / 50.0) as f64, 2))).size(11).color(pal.mute).width(Length::Fill)].spacing(8).wrap().vertical_spacing(4));
                        info = info.push(iced::widget::row![knopf(a.mode.label().into(), vc_i18n::t!("studio-ablauf-schleife-hin-und-zurueck-einmal").into(), true, Aktion::Anim(AnimOp::Mode(match a.mode { sprite_mode::Loop => sprite_mode::PingPong, sprite_mode::PingPong => sprite_mode::Once, sprite_mode::Once => sprite_mode::Loop })), false), knopf(vc_i18n::t!("studio-frame-anhaengen").into(), vc_i18n::t!("studio-das-gewaehlte-frame-als-naechsten-schritt").into(), true, Aktion::Anim(AnimOp::AddStep(d.fi)), false), knopf(vc_i18n::t!("studio-loeschen").into(), vc_i18n::t!("studio-animation-loeschen").into(), true, Aktion::Anim(AnimOp::Delete), false)].spacing(4).wrap());
                        for (i, (f, dauer)) in a.steps.iter().enumerate() {
                            info = info.push(iced::widget::row![text(format!("{}", i + 1)).size(11).color(pal.mute).width(14), button(text(vc_i18n::t!("studio-frame-f", f = &f)).size(11.5).color(pal.text)).padding([2, 6]).on_press(Msg::StudioAkt(r.clone(), Aktion::Frame(*f))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { radius: 4.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }),
                                knopf("−".into(), vc_i18n::t!("studio-dauer-kuerzer").into(), *dauer > 1, Aktion::Anim(AnimOp::Delay(i, dauer.saturating_sub(1))), false), text(vc_i18n::t!("studio-dauer-b", dauer = &dauer)).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text), knopf("+".into(), vc_i18n::t!("studio-dauer-laenger").into(), *dauer < 255, Aktion::Anim(AnimOp::Delay(i, dauer + 1)), false),
                                knopf("◀".into(), vc_i18n::t!("studio-schritt-nach-vorn").into(), i > 0, Aktion::Anim(AnimOp::Move(i, -1)), false), knopf("▶".into(), vc_i18n::t!("studio-schritt-nach-hinten").into(), i + 1 < a.steps.len(), Aktion::Anim(AnimOp::Move(i, 1)), false), knopf("×".into(), vc_i18n::t!("studio-schritt-entfernen").into(), true, Aktion::Anim(AnimOp::DelStep(i)), false)].spacing(3).align_y(Alignment::Center));
                        }
                    }
                }
            }
        }
        // Zone 1: Kopfzeile (alle Datei-Aktionen nur hier), Zone 3: Liste, Zone 4: Fläche, Zone 5: Seitenspalte aus Karten, Zone 6: Fußzeile
        let name = rel.rsplit('/').next().unwrap_or(rel).to_string();
        let art = match d.kind.as_str() { "sprite" => "SPR", "pmg" => "PMG", "charset" => "CHR", _ => "MED" };
        let masse = vc_i18n::t!("studio-frame-p1-von-p2-p3-p4", p1 = &d.fi, p2 = d.frames.len(), p3 = &d.width, p4 = &d.height, p5 = if d.kind == "sprite" { if d.ist_multicolor(d.fi) { " · Multicolor" } else { " · Hires" } } else { "" });
        let kopf = muster::kopfzeile(pal, klasse, art, &name, d.dirty(), &masse, vec![
            knopf(vc_i18n::t!("studio-png-laden").into(), vc_i18n::t!("studio-bild-in-das-gewaehlte-frame-uebernehmen").into(), true, Aktion::PngLaden, false),
            knopf(vc_i18n::t!("studio-exportieren").into(), vc_i18n::t!("studio-export-dialog-cmd-umschalt-e").into(), true, Aktion::Exportieren, false),
            knopf(vc_i18n::t!("studio-speichern").into(), vc_i18n::t!("studio-speichern-cmd-s").into(), d.dirty(), Aktion::Speichern, false)]);
        let karte_zu = |k: &'static str| -> Option<(bool, Msg)> { Some((!s.karten_zu.contains(k), Msg::StudioAkt(r.clone(), Aktion::Karte(k)))) };
        let mut karten: Vec<Element<Msg>> = vec![
            muster::karte(pal, &vc_i18n::t!("studio-werkzeug"), "✎", &vc_i18n::t!("studio-fuers-zeichnen-nicht-in-der-datei"), karte_zu("werkzeug"), iced::widget::column![text(vc_i18n::t!("studio-zeichenfarbe")).size(muster::schrift::HINWEIS).color(pal.mute), werkzeug].spacing(muster::abstand::ENG).into()),
            muster::karte(pal, &vc_i18n::t!("studio-dokument"), "▤", &if d.kind == "sprite" { vc_i18n::t!("studio-fuers-ganze-frame-bzw-sprite-wird-gespeichert") } else { vc_i18n::t!("studio-gilt-fuers-ganze-dokument-wird-gespeichert") }, karte_zu("dokument"), dokument.into())];
        if d.kind == "sprite" || vc_ide_model::pixel::has_sprite_flags(&d.target, &d.kind) { karten.push(muster::karte(pal, &vc_i18n::t!("studio-sprite"), "ⓘ", &vc_i18n::t!("studio-hardware-flags-und-animationen"), karte_zu("info"), info.into())) }
        else if herkunft.is_some() { karten.push(muster::karte(pal, &vc_i18n::t!("studio-herkunft"), "ⓘ", &vc_i18n::t!("studio-woher-dieser-zeichensatz-stammt"), karte_zu("herkunft"), info.into())) }
        let liste_el: Element<Msg> = if klasse == muster::Klasse::Schmal { iced::widget::column![frames].into() } else { scrollable(frames).into() };
        let liste_opt = if klasse == muster::Klasse::Schmal { karten.insert(0, muster::karte(pal, &vc_i18n::t!("studio-frames"), "▦", "", karte_zu("frames"), liste_el)); None } else { Some(liste_el) };
        let seite = muster::seitenspalte(klasse, karten);
        let zelle = s.ueber.map_or(String::new(), |(x, y)| format!("{} ", vc_i18n::t!("studio-zelle-x-y", x = &x, y = &y)));
        let status = if d.status.is_empty() { String::new() } else { format!(" · {}", d.status) };
        let (ew, eh) = einheiten(d);
        let fuss = muster::fusszeile(pal, &vc_i18n::t!("studio-zelle-zoom-p2-status", zelle = &zelle, p2 = s.zoomansicht.zoom_text(ew, eh), status = &status),
            tooltip(text(vc_i18n::t!("studio-tasten")).size(muster::schrift::HINWEIS).color(pal.mute), container(text(vc_i18n::t!("studio-linke-taste-zeichnet-rechte-loescht-d")).size(11.5).color(pal.text)).padding([4, 8]).max_width(420).style(move |_| flaeche(pal.panel2, pal.line)), tooltip::Position::Top).into());
        let flaeche_el: Element<Msg> = canvas::Canvas::new(Flaeche { s, farben: &self.farben, norm, rel: rel.to_string(), pal }).width(Length::Fill).height(Length::Fill).into();
        let flaeche_el: Element<Msg> = if s.mehr {
            let karte_mehr = container(iced::widget::column![if sehr_schmal { Element::from(iced::widget::column![text(vc_i18n::t!("studio-werkzeuge")).size(muster::schrift::HINWEIS).color(pal.mute), werkzeuge_b()].spacing(muster::abstand::ENG)) } else { space::vertical().height(0).into() }, text(vc_i18n::t!("studio-transformieren-2")).size(muster::schrift::HINWEIS).color(pal.mute), iced::widget::row![
                knopf("⇋".into(), vc_i18n::t!("studio-horizontal-spiegeln-x").into(), true, Aktion::Spiegeln("mh"), false), knopf("⇅".into(), vc_i18n::t!("studio-vertikal-spiegeln-y").into(), true, Aktion::Spiegeln("mv"), false),
                knopf("←".into(), vc_i18n::t!("studio-nach-links-verschieben").into(), true, Aktion::Verschieben("sl"), false), knopf("→".into(), vc_i18n::t!("studio-nach-rechts-verschieben").into(), true, Aktion::Verschieben("sr"), false), knopf("↑".into(), vc_i18n::t!("studio-nach-oben-verschieben").into(), true, Aktion::Verschieben("su"), false), knopf("↓".into(), vc_i18n::t!("studio-nach-unten-verschieben").into(), true, Aktion::Verschieben("sd"), false),
                knopf(vc_i18n::t!("studio-leeren").into(), vc_i18n::t!("studio-frame-leeren").into(), true, Aktion::Leeren, false)].spacing(4).wrap().vertical_spacing(4),
                if klasse == muster::Klasse::Breit { Element::from(space::vertical().height(0)) } else { text(vc_i18n::t!("studio-ansicht")).size(muster::schrift::HINWEIS).color(pal.mute).into() }, if klasse == muster::Klasse::Breit { Element::from(space::vertical().height(0)) } else { iced::widget::row![knopf("−".into(), vc_i18n::t!("studio-verkleinern").into(), true, Aktion::ZoomMinus, false), knopf("+".into(), vc_i18n::t!("studio-vergroessern").into(), true, Aktion::ZoomPlus, false), knopf(vc_i18n::t!("studio-einpassen").into(), vc_i18n::t!("studio-flaeche-einpassen-0").into(), true, Aktion::ZoomAnpassen, s.zoomansicht.zoom.is_none()), knopf(vc_i18n::t!("studio-raster-2").into(), vc_i18n::t!("studio-raster-ein-aus").into(), true, Aktion::Raster, s.raster), knopf(vc_i18n::t!("studio-onion-skin-2").into(), vc_i18n::t!("studio-vorheriges-frame-durchscheinen-lassen").into(), d.fi > 0, Aktion::Zwiebel, s.zwiebel)].spacing(4).wrap().vertical_spacing(4).into() }].spacing(muster::abstand::GRUPPE))
                .padding(muster::abstand::KARTE).max_width(340).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: muster::RADIUS_KARTE.into() }, ..Default::default() });
            iced::widget::stack![flaeche_el, container(karte_mehr).padding(muster::abstand::GRUPPE).align_right(Length::Fill)].into()
        } else { flaeche_el };
        // Vorschau 1:1 links unten auf der Fläche (Sprite, Player/Missile): so groß, wie es im Spiel erscheint
        let flaeche_el: Element<Msg> = if d.kind == "sprite" || d.kind == "pmg" {
            let vorschau = container(iced::widget::column![canvas::Canvas::new(Vorschau { doc: d, fi: d.fi, farben: &self.farben, norm, pal }).width(Length::Fixed(if d.kind == "sprite" { 24.0 } else { d.width as f32 })).height(Length::Fixed(d.height as f32)), text(vc_i18n::t!("studio-vorschau-1")).size(muster::schrift::HINWEIS - 1.0).color(pal.mute)].spacing(2).align_x(Alignment::Center))
                .padding(6).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: muster::RADIUS_KNOPF.into() }, ..Default::default() });
            iced::widget::stack![flaeche_el, container(vorschau).padding(muster::abstand::GRUPPE).align_left(Length::Fill).align_bottom(Length::Fill)].into()
        } else { flaeche_el };
        let flaeche_el: Element<Msg> = container(flaeche_el).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), ..Default::default() }).into();
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: if d.kind == "charset" { 270.0 } else { muster::LISTE_BREITE }, kopf, leiste, liste: liste_opt, flaeche: flaeche_el, seite, fuss })
    }
}
