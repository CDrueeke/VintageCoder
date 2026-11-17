// SPDX-License-Identifier: GPL-3.0-or-later
//! Ansicht der Debug-Sitzung (Entscheidung 23): Symbolleiste mit gezeichneten Symbolen, Register als Tabelle, Flags als Bit-Anzeige,
//! Disassembler mit Haltepunkt-Rand, Speicher mit ASCII, Beobachten, Haltepunkte, Momentaufnahme. Die Bausteine sind unabhängig von
//! der Anordnung (Panel, Debug-Fläche in der Schublade).
use super::*;
use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{rich_text, span};
use vc_gui_ui::fonts::{MONO, MONO_FETT};

/// Symbole der Symbolleiste (gezeichnet, keine Schriftzeichen: keine Rückfallschrift, kein Emoji)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ikon { Weiter, Pause, Ueber, Hinein, Hinaus, Ende, Kaefer }

struct IkonZeichnung { art: Ikon, farbe: Color }
impl<M> canvas::Program<M> for IkonZeichnung {
    type State = ();
    fn draw(&self, _s: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = Frame::new(r, b.size()); let k = b.width / 16.0; let p = |x: f32, y: f32| iced::Point::new(x * k, y * k);
        let strich = Stroke::default().with_color(self.farbe).with_width(1.7 * k).with_line_cap(canvas::LineCap::Round).with_line_join(canvas::LineJoin::Round);
        let fuellen = |f: &mut Frame, pfad: &Path| f.fill(pfad, self.farbe);
        match self.art {
            Ikon::Weiter => fuellen(&mut f, &Path::new(|b| { b.move_to(p(4.0, 2.5)); b.line_to(p(13.5, 8.0)); b.line_to(p(4.0, 13.5)); b.close() })),
            Ikon::Pause => { fuellen(&mut f, &Path::rectangle(p(3.5, 3.0), iced::Size::new(3.2 * k, 10.0 * k))); fuellen(&mut f, &Path::rectangle(p(9.3, 3.0), iced::Size::new(3.2 * k, 10.0 * k))) }
            Ikon::Kaefer => {
                // Käfer: Körper, Kopf, drei Beinpaare, zwei Fühler
                fuellen(&mut f, &Path::circle(p(8.0, 9.5), 3.6 * k)); fuellen(&mut f, &Path::circle(p(8.0, 4.6), 2.0 * k));
                for (y, dx) in [(7.5, 5.8), (10.0, 6.2), (12.5, 5.6)] { f.stroke(&Path::line(p(8.0 - 3.0, y), p(8.0 - dx, y + if y < 9.0 { -1.2 } else { 1.0 })), strich); f.stroke(&Path::line(p(8.0 + 3.0, y), p(8.0 + dx, y + if y < 9.0 { -1.2 } else { 1.0 })), strich) }
                f.stroke(&Path::line(p(7.0, 3.2), p(5.5, 1.5)), strich); f.stroke(&Path::line(p(9.0, 3.2), p(10.5, 1.5)), strich)
            }
            Ikon::Ende => fuellen(&mut f, &Path::rectangle(p(3.0, 3.0), iced::Size::new(10.0 * k, 10.0 * k))),
            Ikon::Ueber => { f.stroke(&Path::new(|b| { b.move_to(p(2.5, 9.5)); b.quadratic_curve_to(p(8.0, -2.5), p(13.0, 8.0)) }), strich); fuellen(&mut f, &Path::new(|b| { b.move_to(p(10.2, 7.6)); b.line_to(p(14.4, 7.8)); b.line_to(p(12.8, 11.6)); b.close() })); fuellen(&mut f, &Path::circle(p(8.0, 13.0), 1.6 * k)) }
            Ikon::Hinein => { f.stroke(&Path::line(p(8.0, 1.5), p(8.0, 9.5)), strich); fuellen(&mut f, &Path::new(|b| { b.move_to(p(4.0, 7.5)); b.line_to(p(12.0, 7.5)); b.line_to(p(8.0, 12.0)); b.close() })); f.stroke(&Path::line(p(3.0, 14.5), p(13.0, 14.5)), strich) }
            Ikon::Hinaus => { f.stroke(&Path::line(p(8.0, 14.5), p(8.0, 6.5)), strich); fuellen(&mut f, &Path::new(|b| { b.move_to(p(4.0, 8.5)); b.line_to(p(12.0, 8.5)); b.line_to(p(8.0, 4.0)); b.close() })); f.stroke(&Path::line(p(3.0, 1.5), p(13.0, 1.5)), strich) }
        }
        vec![f.into_geometry()]
    }
}
pub fn ikon<'a>(art: Ikon, farbe: Color) -> Element<'a, Msg> { canvas::Canvas::new(IkonZeichnung { art, farbe }).width(16).height(16).into() }


/// Eine Spalte der Debug-Fläche: eigene Bildlaufleiste, Rand für die Leiste
fn spalte<'a>(inhalt: Element<'a, Msg>, breite: Length) -> Element<'a, Msg> { container(scrollable(container(inhalt).padding(iced::Padding { top: 6.0, right: 14.0, bottom: 8.0, left: 12.0 })).width(Length::Fill).height(Length::Fill)).width(breite).height(Length::Fill).into() }

/// Ein Zeichen für die ASCII-Spalte des Speichers
fn ascii(b: u8) -> char { if (0x20..0x7F).contains(&b) { b as char } else { '·' } }

impl App {
    /// Befehl der Symbolleiste zu einem Symbol
    pub(crate) fn ikon_befehl(a: Ikon) -> Cmd { match a { Ikon::Weiter => Cmd::DebugWeiter, Ikon::Pause => Cmd::DebugPause, Ikon::Ueber => Cmd::DebugUeber, Ikon::Hinein => Cmd::DebugHinein, Ikon::Hinaus => Cmd::DebugRaus, Ikon::Ende => Cmd::DebugEnde, Ikon::Kaefer => Cmd::Debuggen } }

    /// Ein Knopf der Symbolleiste: gezeichnetes Symbol, Hinweis mit Name und Kürzel, ausgegraut wenn der Befehl jetzt nicht geht
    fn ikon_knopf(&self, pal: Pal, a: Ikon) -> Element<'_, Msg> {
        let c = Self::ikon_befehl(a); let info = befehle::info(c);
        let an = self.nutzbar(c) && !self.snap.starting;
        let farbe = if !an { pal.mute } else { match a { Ikon::Weiter => pal.ok, Ikon::Ende => pal.err, _ => pal.text } };
        let mut b = button(container(ikon(a, farbe)).center_x(Length::Shrink)).padding([6, 9]).style(move |_, st| button::Style { background: if an && matches!(st, button::Status::Hovered) { Some(pal.sel.into()) } else { None }, border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() });
        if an { b = b.on_press(Msg::Befehl(c)) }
        let k = befehle::fuer_anzeige(info.tasten, cfg!(target_os = "macos"));
        let tipp = container(iced::widget::row![text(info.name).size(12).color(pal.text), text(k).size(11).color(pal.mute)].spacing(10)).padding([4, 8]).style(move |_| flaeche(pal.panel2, pal.line));
        iced::widget::tooltip(b, tipp, iced::widget::tooltip::Position::Bottom).gap(4).into()
    }
    /// Symbolleiste für die Kopfzeile: drei Gruppen (Weiter/Anhalten · Über/Hinein/Hinaus · Beenden) und der Zustand als Text
    pub(crate) fn debug_leiste(&self, pal: Pal) -> Element<'_, Msg> {
        let gruppe = |ik: &[Ikon]| -> Element<Msg> { container(row(ik.iter().map(|a| self.ikon_knopf(pal, *a)).collect::<Vec<_>>()).spacing(0)).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 8.0.into() }, ..Default::default() }).into() };
        iced::widget::row![gruppe(&[Ikon::Weiter, Ikon::Pause]), gruppe(&[Ikon::Ueber, Ikon::Hinein, Ikon::Hinaus]), gruppe(&[Ikon::Ende]), text(self.debug_status_kurz()).size(12.5).color(pal.text)].spacing(8).align_y(Alignment::Center).into()
    }
    /// Schmale Form: nur die Symbolgruppen, der Zustand steht im Tooltip
    pub(crate) fn debug_leiste_kompakt(&self, pal: Pal) -> Element<'_, Msg> {
        let gruppe = |ik: &[Ikon]| -> Element<Msg> { container(row(ik.iter().map(|a| self.ikon_knopf(pal, *a)).collect::<Vec<_>>()).spacing(0)).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 8.0.into() }, ..Default::default() }).into() };
        let leiste = iced::widget::row![gruppe(&[Ikon::Weiter, Ikon::Pause]), gruppe(&[Ikon::Ueber, Ikon::Hinein, Ikon::Hinaus]), gruppe(&[Ikon::Ende])].spacing(6).align_y(Alignment::Center);
        iced::widget::tooltip(leiste, container(text(self.debug_status_kurz()).size(12).color(pal.text)).padding([4, 8]).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, ..Default::default() }, ..Default::default() }), iced::widget::tooltip::Position::Bottom).into()
    }
    /// Kurzform für die Kopfzeile
    pub(crate) fn debug_status_kurz(&self) -> String { let sn = &self.snap; if sn.starting { "startet …".into() } else if sn.running { "läuft".into() } else if sn.have_regs { format!("angehalten · ${:04X}", sn.pc) } else { "wartet …".into() } }
    /// Zustand der Sitzung als Satz
    pub(crate) fn debug_status(&self) -> String {
        let sn = &self.snap;
        if sn.starting { t::DBG_STARTET.to_string() } else if sn.running { t::DBG_LAEUFT.to_string() } else if sn.have_regs {
            let ort = self.info.as_ref().and_then(|i| i.zeile_bei(sn.pc)).map(|(f, n)| format!(" · {f}:{n}")).unwrap_or_default();
            format!("{} ${:04X}{ort}", t::DBG_ANGEHALTEN, sn.pc)
        } else { t::DBG_WARTET.to_string() }
    }
    /// Abschnittstitel mit Trennlinie (Hierarchie), rechts ein Zusatz
    fn abschnitt<'a>(&self, pal: Pal, titel: &str, rechts: &str) -> Element<'a, Msg> {
        iced::widget::column![row![text(titel.to_string()).size(10.5).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute), space::horizontal(), text(rechts.to_string()).size(10.5).color(pal.mute)], container(space::horizontal()).height(1).width(Length::Fill).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() })].spacing(4).width(Length::Fill).into()
    }
    /// Satz der Register, wie er jetzt gilt: PC, A, X, Y, SP, Flags
    fn reg_jetzt(&self) -> [u16; 6] { let s = &self.snap; [s.pc, s.a as u16, s.x as u16, s.y as u16, s.sp as u16, s.flags as u16] }

    /// Register als Tabelle (Name · Hex · Dezimal · „war …“), geänderte Werte hervorgehoben, darunter die Flags als Bit-Anzeige
    pub(crate) fn dbg_register(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap; let jetzt = self.reg_jetzt(); let alt = self.reg_alt;
        let mut c = iced::widget::column![self.abschnitt(pal, t::DBG_REGISTER, "Hex · Dez")].spacing(4);
        if !sn.have_regs { return c.push(text(if sn.running { t::DBG_REG_LAEUFT } else { t::DBG_WARTET }).size(12).color(pal.mute)).into() }
        for (i, n) in ["PC", "A", "X", "Y", "SP"].iter().enumerate() {
            let v = jetzt[i]; let breit = i == 0; let geaendert = alt.is_some_and(|a| a[i] != v);
            let hex = if breit { format!("{v:04X}") } else { format!("{v:02X}") }; let war = alt.filter(|a| a[i] != v).map(|a| if breit { format!("war {:04X}", a[i]) } else { format!("war {:02X}", a[i]) }).unwrap_or_default();
            let zeile = iced::widget::row![text(*n).size(12).color(pal.mute).width(30), text(hex).size(12.5).font(if geaendert { MONO_FETT } else { MONO }).color(if geaendert { pal.accent } else { pal.text }).width(46).align_x(iced::alignment::Horizontal::Right), text(format!("{v}")).size(11.5).font(MONO).color(pal.mute).width(44).align_x(iced::alignment::Horizontal::Right), text(war).size(10.5).font(MONO).color(pal.mute)].spacing(8).align_y(Alignment::Center);
            c = c.push(container(zeile).padding([1, 4]).width(Length::Fill).style(move |_| container::Style { background: geaendert.then(|| Color { a: 0.14, ..pal.accent }.into()), border: Border { radius: 4.0.into(), ..Border::default() }, ..Default::default() }));
        }
        // Flags als Bit-Anzeige: NV-BDIZC, gesetzt gefüllt, geändert umrandet
        let mut fl = iced::widget::row![].spacing(2);
        for (i, ch) in "NV-BDIZC".chars().enumerate() {
            let an = jetzt[5] & (0x80 >> i) != 0; let leer = ch == '-'; let geaendert = alt.is_some_and(|a| (a[5] ^ jetzt[5]) & (0x80 >> i) != 0);
            fl = fl.push(container(text(ch.to_string()).size(11).font(MONO_FETT).color(if an && !leer { pal.accent_text() } else if leer { pal.line } else { pal.mute })).center_x(Length::Fill).center_y(Length::Fill).width(18).height(18).style(move |_| container::Style { background: Some(if an && !leer { pal.accent.into() } else { pal.panel2.into() }), border: Border { color: if geaendert { pal.accent } else { pal.line }, width: if geaendert { 2.0 } else { 1.0 }, radius: 4.0.into() }, ..Default::default() }));
        }
        c = c.push(container(fl).padding([4, 4]));
        if !sn.extra.is_empty() { c = c.push(text(sn.extra.iter().map(|(n, v)| format!("{n} {v:X}")).collect::<Vec<_>>().join("  ")).size(11).font(MONO).color(pal.mute)) }
        c.into()
    }

    /// Disassembler mit Rand (Haltepunkt, Programmzähler), festen Spalten und Zyklen rechts; Klick auf den Rand setzt einen Haltepunkt
    pub(crate) fn dbg_disassembler(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap; let mut c = iced::widget::column![self.abschnitt(pal, t::DBG_DISASSEMBLER, "Zyklen")].spacing(1);
        if sn.disasm.is_empty() || sn.running { return c.push(text(if sn.running { t::DBG_REG_LAEUFT } else { t::DBG_WARTET }).size(12).color(pal.mute)).into() }
        let bps = self.gewuenschte_haltepunkte();
        for l in &sn.disasm {
            let aktuell = l.addr == sn.pc; let bp = bps.contains(&l.addr) || sn.points.iter().any(|p| p.kind == "exec" && p.start == l.addr);
            if let Some(m) = self.info.as_ref().and_then(|i| i.marke_bei(l.addr)) { c = c.push(iced::widget::row![space::horizontal().width(40), text(format!("{m}:")).size(12).font(MONO_FETT).color(pal.lab)]) }
            let (mn, op) = match l.text.split_once(' ') { Some((a, b)) => (a.to_string(), b.to_string()), None => (l.text.clone(), String::new()) };
            let bytes = l.bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
            let rand = button(text(if bp { "●" } else if aktuell { "▶" } else { " " }).size(11).color(if bp { pal.err } else { pal.accent }).center()).width(22).padding([1, 0]).on_press(Msg::DisHaltepunkt(l.addr)).style(move |_, st| button::Style { background: matches!(st, button::Status::Hovered).then(|| Color { a: 0.25, ..pal.err }.into()), text_color: pal.text, border: Border { radius: 3.0.into(), ..Border::default() }, ..Default::default() });
            let zyk = if l.cycles > 0 { format!("{}{}", l.cycles, if l.extra { "+" } else { "" }) } else { String::new() };
            let zeile = iced::widget::row![rand, text(format!("{:04X}", l.addr)).size(12).font(MONO).color(pal.mute).width(42), text(bytes).size(12).font(MONO).color(pal.mute).width(70), text(mn).size(12).font(MONO_FETT).color(if l.illegal { pal.warn } else { pal.kw }).width(40), text(op).size(12).font(MONO).color(pal.num).width(Length::Fill), text(zyk).size(11).font(MONO).color(pal.mute).width(26).align_x(iced::alignment::Horizontal::Right)].spacing(4).align_y(Alignment::Center);
            c = c.push(container(zeile).width(Length::Fill).style(move |_| container::Style { background: aktuell.then(|| Color { a: 0.22, ..pal.accent }.into()), border: Border { radius: 3.0.into(), ..Border::default() }, ..Default::default() }));
        }
        c.into()
    }

    /// Speicher: Adressfeld, je Zeile 8 Byte in zwei Vierergruppen und ASCII; geänderte Bytes in Akzentfarbe und fett
    pub(crate) fn dbg_speicher(&self, pal: Pal, zeilen: usize) -> Element<'_, Msg> {
        let sn = &self.snap; let mut c = iced::widget::column![self.abschnitt(pal, t::DBG_SPEICHER_TITEL, "Hex · ASCII")].spacing(2);
        c = c.push(text_input(t::DBG_SPEICHER_FELD, &self.mem_text).on_input(Msg::MemText).on_submit(Msg::MemGo).size(12).padding(4).font(MONO));
        if sn.running || sn.mem.is_empty() { return c.push(text(if sn.running { t::DBG_REG_LAEUFT } else { t::DBG_WARTET }).size(12).color(pal.mute)).into() }
        for (r, z) in sn.mem.chunks(8).take(zeilen).enumerate() {
            let base = (sn.mem_addr as usize + r * 8) & 0xFFFF;
            let mut sp: Vec<iced::widget::text::Span<'static, (), iced::Font>> = vec![span(format!("{base:04X}  ")).font(MONO).color(pal.mute)];
            for (i, b) in z.iter().enumerate() {
                let g = sn.mem_prev.get(r * 8 + i).is_some_and(|p| p != b);
                sp.push(span(format!("{b:02X}{}", if i == 3 { "  " } else { " " })).font(if g { MONO_FETT } else { MONO }).color(if g { pal.accent } else { pal.text }));
            }
            let asc: String = z.iter().map(|b| ascii(*b)).collect();
            c = c.push(iced::widget::row![rich_text(sp).size(12), text(format!(" {asc}")).size(12).font(MONO).color(pal.mute).shaping(iced::widget::text::Shaping::Basic).wrapping(iced::widget::text::Wrapping::None)]);
        }
        c.into()
    }

    /// Beobachten: Eingabe, je Ausdruck eine Zeile „Ausdruck … Wert“, Fehler kurz und umbrechend darunter
    pub(crate) fn dbg_beobachten(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap; let mut c = iced::widget::column![self.abschnitt(pal, t::DBG_BEOBACHTEN, "Dez · Hex")].spacing(3);
        c = c.push(text_input(t::DBG_AUSDRUCK, &self.beob_text).on_input(Msg::BeobText).on_submit(Msg::BeobAdd).size(12).padding(4).font(MONO));
        for (i, e) in self.beob.iter().enumerate() {
            let w = sn.watch.get(i).filter(|(n, _)| n == e).map(|(_, w)| w);
            let wert = match w { Some(Ok(v)) => format!("{v} · ${v:X}"), Some(Err(_)) => "unbekannt".to_string(), None => "…".to_string() };
            let farbe = match w { Some(Ok(_)) => pal.text, Some(Err(_)) => pal.err, None => pal.mute };
            c = c.push(iced::widget::row![text(e.clone()).size(12).font(MONO_FETT).color(pal.text), space::horizontal(), text(wert).size(12).font(MONO).color(farbe), button(text("✕").size(11).color(pal.mute)).padding([0, 4]).on_press(Msg::BeobDel(i)).style(move |_, _| button::Style { background: None, text_color: pal.mute, ..Default::default() })].spacing(6).align_y(Alignment::Center));
            if let Some(Err(er)) = w { c = c.push(text(er.clone()).size(10.5).color(pal.err).width(Length::Fill)) }
        }
        c.into()
    }

    /// Haltepunkte der Sitzung: Art, Ort, Bedingung, Treffer; ✕ löscht
    pub(crate) fn dbg_haltepunkte(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap; let mut c = iced::widget::column![self.abschnitt(pal, t::DBG_HALTEPUNKTE, "Treffer")].spacing(3);
        if sn.points.is_empty() { return c.push(text(t::DBG_KEINE_HALTEPUNKTE).size(11.5).color(pal.mute).width(Length::Fill)).into() }
        for p in &sn.points {
            let ort = if p.start == p.end { format!("${:04X}", p.start) } else { format!("${:04X}–${:04X}", p.start, p.end) };
            let zeile = self.info.as_ref().and_then(|i| i.zeile_bei(p.start)).map(|(f, n)| format!(" · {f}:{n}")).unwrap_or_default();
            c = c.push(iced::widget::row![text("●").size(10).color(pal.err), text(format!("{} {ort}{zeile}{}", p.kind, p.cond.as_ref().map(|c| format!(" wenn {c}")).unwrap_or_default())).size(12).font(MONO).color(pal.text).width(Length::Fill), text(format!("{}×", p.hits)).size(11).font(MONO).color(pal.mute), button(text("✕").size(11).color(pal.mute)).padding([0, 4]).on_press(Msg::PunktDel(p.id)).style(move |_, _| button::Style { background: None, text_color: pal.mute, ..Default::default() })].spacing(6).align_y(Alignment::Center));
        }
        c.into()
    }

    /// Momentaufnahme (nur angehalten)
    pub(crate) fn dbg_momentaufnahme(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap; let mono = |s: String, c: Color| -> Element<Msg> { text(s).size(11.5).font(MONO).color(c).into() };
        let mut c = iced::widget::column![self.abschnitt(pal, t::DBG_MOMENT, "")].spacing(4);
        if sn.running || !sn.have_regs { return c.push(text(t::DBG_REG_LAEUFT).size(12).color(pal.mute)).into() }
        c = c.push(button(text(t::DBG_MOMENT_ERKENNEN).size(11.5).color(pal.text)).padding([3, 8]).on_press(Msg::SnapErkennen).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }));
        match &sn.detected {
            Some(Err(e)) => c = c.push(text(e.clone()).size(11).color(pal.warn).width(Length::Fill)),
            Some(Ok(det)) => {
                c = c.push(mono(format!("{} · Bildschirm ${:04X}", if det.mode.is_empty() { det.system.as_str() } else { det.mode.as_str() }, det.screen), pal.text));
                c = c.push(mono(match (det.charset, det.rom_off) { (Some(a), _) => format!("Zeichensatz ${a:04X}{}", if det.charset_multicolor { " (mehrfarbig)" } else { "" }), (None, Some(_)) => "Zeichensatz im ROM".into(), _ => "Zeichensatz unbekannt".into() }, pal.mute));
                c = c.push(mono(format!("{} Sprite(s)", det.sprites.len()), pal.mute));
                for n in &det.notes { c = c.push(text(n.clone()).size(10.5).color(pal.mute).width(Length::Fill)) }
                c = c.push(iced::widget::checkbox(self.snap_sprites).label(t::DBG_MOMENT_SPRITES).on_toggle(Msg::SnapSprites).size(14).text_size(11.5));
                c = c.push(iced::widget::checkbox(self.snap_tiles).label(t::DBG_MOMENT_KACHELN).on_toggle(Msg::SnapKacheln).size(14).text_size(11.5));
                if det.rom_off.is_some() && det.charset.is_none() { c = c.push(iced::widget::checkbox(self.snap_rom).label(t::DBG_MOMENT_ROM).on_toggle(Msg::SnapRom).size(14).text_size(11.5)) }
                c = c.push(text_input(t::DBG_MOMENT_NAME, &self.snap_name).on_input(Msg::SnapName).on_submit(Msg::SnapSpeichern).size(12).padding(4));
                c = c.push(button(text(t::DBG_MOMENT_SPEICHERN).size(11.5).color(pal.text)).padding([3, 8]).on_press(Msg::SnapSpeichern).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }));
            }
            None => {}
        }
        c.into()
    }

    /// Panel „Debug“: Zustand, Register, Disassembler, Speicher, Beobachten, Haltepunkte, Momentaufnahme (eine Spalte)
    pub(crate) fn panel_debug(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap;
        let mut c = iced::widget::column![text(self.debug_status()).size(12.5).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).width(Length::Fill)].spacing(10).width(Length::Fill);
        if let Some(b) = &sn.step_block { c = c.push(text(b.clone()).size(11).color(pal.warn).width(Length::Fill)) }
        c = c.push(self.dbg_register(pal)).push(self.dbg_disassembler(pal)).push(self.dbg_speicher(pal, 8)).push(self.dbg_beobachten(pal)).push(self.dbg_haltepunkte(pal)).push(self.dbg_momentaufnahme(pal));
        scrollable(c.padding(iced::Padding { right: 14.0, ..Default::default() })).height(Length::Fill).into()
    }

    /// Debug-Fläche (Reiter „Debug“ der Schublade): Zustandszeile, darunter Spalten Register · Disassembler · Speicher · Beobachten/Haltepunkte;
    /// in schmalen Fenstern (< 900) einspaltig mit Reitern
    pub(crate) fn debug_flaeche(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap;
        let trenner = || -> Element<Msg> { container(space::vertical()).width(1).height(Length::Fill).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() }).into() };
        let rest = || -> Element<'_, Msg> { iced::widget::column![self.dbg_beobachten(pal), self.dbg_haltepunkte(pal), self.dbg_momentaufnahme(pal)].spacing(12).into() };
        let mut zeile = iced::widget::row![text(self.debug_status()).size(12.5).font(vc_gui_ui::fonts::UI_FETT).color(pal.text)].spacing(12).align_y(Alignment::Center);
        if let Some(b) = &sn.step_block { zeile = zeile.push(text(b.clone()).size(11.5).color(pal.warn)) }
        let kopf = container(zeile).padding([5, 12]).width(Length::Fill);
        let inhalt: Element<Msg> = if self.schmal() {
            let reiter = ["Register", "Disassembler", "Speicher", "Beobachten"];
            let leiste = iced::widget::row(reiter.iter().enumerate().map(|(i, n)| reiter_knopf(pal, n, self.debug_spalte == i, Msg::DebugSpalte(i))).collect::<Vec<_>>()).spacing(4);
            let b: Element<Msg> = match self.debug_spalte { 0 => self.dbg_register(pal), 1 => self.dbg_disassembler(pal), 2 => self.dbg_speicher(pal, 8), _ => rest() };
            iced::widget::column![container(leiste).padding([2, 8]), spalte(b, Length::Fill)].into()
        } else {
            iced::widget::row![spalte(self.dbg_register(pal), Length::Fixed(192.0)), trenner(), spalte(self.dbg_disassembler(pal), Length::FillPortion(3)), trenner(), spalte(self.dbg_speicher(pal, 8), Length::Fixed(340.0)), trenner(), spalte(rest(), Length::Fixed(184.0))].into()
        };
        iced::widget::column![kopf, inhalt].into()
    }
}
