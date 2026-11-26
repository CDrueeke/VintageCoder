// SPDX-License-Identifier: GPL-3.0-or-later
//! Zahlenwerkzeuge (Befehlspalette „Zahlenwerkzeuge …“): eine Zahl oder ein Ausdruck (`$D020`, `0xFF`, `%1010`, `0b1010`, `255`, `$D000+$20`, `1<<5`, `-1`) wird in allen
//! Schreibweisen gezeigt (dezimal, hexadezimal mit `$`, binär mit `%`, vorzeichenbehaftet, Lo-/Hi-Byte, Zeichen), ein Bit-Editor (8 oder 16 Bit) schaltet Bits per Klick,
//! und jeder Wert lässt sich kopieren oder in den Editor einsetzen. Nur Rechnen mit ganzen Zahlen (64 Bit); keine Fließkommazahlen.
use super::*;
use iced::widget::column;

#[derive(Clone, Debug, PartialEq)]
pub enum ZAkt { Text(String), Bit(u8), Breite(u8), Kopieren(&'static str), Einsetzen(&'static str), Zu }

pub struct ZahlenDlg { pub text: String, pub breite: u8 }
impl Default for ZahlenDlg { fn default() -> Self { ZahlenDlg { text: "$D020".into(), breite: 8 } } }

// ---------------------------------------------------------------------------------------------------- Ausdruck lesen
struct P<'a> { s: &'a [u8], i: usize }
impl<'a> P<'a> {
    fn blank(&mut self) { while self.s.get(self.i).is_some_and(|c| c.is_ascii_whitespace()) { self.i += 1 } }
    fn peek(&mut self) -> Option<u8> { self.blank(); self.s.get(self.i).copied() }
    fn zahl(&mut self) -> Result<i64, String> {
        self.blank(); let start = self.i;
        let (basis, ziffern): (u32, &dyn Fn(u8) -> bool) = if self.s.get(self.i) == Some(&b'$') { self.i += 1; (16, &|c: u8| c.is_ascii_hexdigit() || c == b'_') }
            else if self.s[self.i..].starts_with(b"0x") || self.s[self.i..].starts_with(b"0X") { self.i += 2; (16, &|c: u8| c.is_ascii_hexdigit() || c == b'_') }
            else if self.s.get(self.i) == Some(&b'%') { self.i += 1; (2, &|c: u8| c == b'0' || c == b'1' || c == b'_') }
            else if self.s[self.i..].starts_with(b"0b") || self.s[self.i..].starts_with(b"0B") { self.i += 2; (2, &|c: u8| c == b'0' || c == b'1' || c == b'_') }
            else { (10, &|c: u8| c.is_ascii_digit() || c == b'_') };
        let von = self.i; while self.s.get(self.i).is_some_and(|c| ziffern(*c)) { self.i += 1 }
        let t: String = self.s[von..self.i].iter().filter(|c| **c != b'_').map(|c| *c as char).collect();
        if t.is_empty() { return Err(vc_i18n::t!("zahlen-zahl-erwartet-bei-zeichen-p1", p1 = start + 1)) }
        i64::from_str_radix(&t, basis).map_err(|_| vc_i18n::t!("zahlen-zahl-zu-gross").to_string())
    }
    fn atom(&mut self) -> Result<i64, String> {
        match self.peek() {
            Some(b'(') => { self.i += 1; let v = self.ausdruck(0)?; if self.peek() == Some(b')') { self.i += 1; Ok(v) } else { Err(vc_i18n::t!("zahlen-fehlt").into()) } }
            Some(b'-') => { self.i += 1; Ok(self.atom()?.wrapping_neg()) } Some(b'+') => { self.i += 1; self.atom() } Some(b'~') => { self.i += 1; Ok(!self.atom()?) }
            Some(_) => self.zahl(), None => Err(vc_i18n::t!("zahlen-zahl-erwartet").into()),
        }
    }
    /// Operatoren: `|` 1, `^` 2, `&` 3, `<<` `>>` 4, `+` `-` 5, `*` `/` `%` 6 (kleiner Rang bindet schwächer); `%` ist hier der Rest nur nach einem Wert mit Leerzeichen, sonst Binärpräfix
    fn ausdruck(&mut self, min: u8) -> Result<i64, String> {
        let mut l = self.atom()?;
        loop {
            let Some(c) = self.peek() else { break };
            let (op, rang, laenge): (&str, u8, usize) = match c { b'|' => ("|", 1, 1), b'^' => ("^", 2, 1), b'&' => ("&", 3, 1), b'+' => ("+", 5, 1), b'-' => ("-", 5, 1), b'*' => ("*", 6, 1), b'/' => ("/", 6, 1),
                b'<' if self.s.get(self.i + 1) == Some(&b'<') => ("<<", 4, 2), b'>' if self.s.get(self.i + 1) == Some(&b'>') => (">>", 4, 2), _ => break };
            if rang < min { break }
            self.i += laenge; let r = self.ausdruck(rang + 1)?;
            l = match op { "|" => l | r, "^" => l ^ r, "&" => l & r, "+" => l.wrapping_add(r), "-" => l.wrapping_sub(r), "*" => l.wrapping_mul(r),
                "/" => { if r == 0 { return Err(vc_i18n::t!("zahlen-teilung-durch-0").into()) } l / r } "<<" => l.checked_shl(r as u32).unwrap_or(0), ">>" => l.checked_shr(r as u32).unwrap_or(0), _ => l };
        }
        Ok(l)
    }
}
/// Wert eines Ausdrucks (ganze Zahlen)
pub fn werte(text: &str) -> Result<i64, String> {
    let mut p = P { s: text.trim().as_bytes(), i: 0 }; if p.s.is_empty() { return Err(vc_i18n::t!("zahlen-zahl-oder-ausdruck-eingeben").into()) }
    let v = p.ausdruck(0)?; if p.peek().is_some() { return Err(vc_i18n::t!("zahlen-unerwartetes-zeichen-bei-p1", p1 = p.i + 1)) } Ok(v)
}

// ---------------------------------------------------------------------------------------------------- Schreibweisen
pub fn hex(v: i64) -> String { if (0..=0xFF).contains(&v) { format!("${v:02X}") } else if (0..=0xFFFF).contains(&v) { format!("${v:04X}") } else if v >= 0 { format!("${v:X}") } else { format!("-${:X}", v.unsigned_abs()) } }
pub fn binaer(v: i64) -> String { let b = if (0..=0xFF).contains(&v) { 8 } else if (0..=0xFFFF).contains(&v) { 16 } else { 0 }; if v < 0 { format!("-%{:b}", v.unsigned_abs()) } else if b > 0 { format!("%{v:0b$b}") } else { format!("%{v:b}") } }
/// Wert als Zweierkomplement in `bits` Bit (8 oder 16)
pub fn vorzeichen(v: i64, bits: u8) -> i64 { let m = (1i64 << bits) - 1; let u = v & m; if u >> (bits - 1) == 1 { u - (1 << bits) } else { u } }
pub fn zeichen(v: i64) -> Option<char> { (0x20..0x7F).contains(&v).then(|| v as u8 as char) }
/// Bit `n` umschalten (nur innerhalb der gewählten Breite)
pub fn bit_umschalten(v: i64, n: u8, breite: u8) -> i64 { if n >= breite { return v } let m = (1i64 << breite) - 1; (v & !m) | ((v & m) ^ (1 << n)) }

impl App {
    pub(crate) fn zahlen_aktion(&mut self, a: ZAkt) -> Task<Msg> {
        let Some(d) = self.zahlen.as_mut() else { return Task::none() };
        match a {
            ZAkt::Text(t) => d.text = t, ZAkt::Breite(b) => d.breite = b, ZAkt::Zu => self.zahlen = None,
            ZAkt::Bit(n) => { let (b, t) = (d.breite, d.text.clone()); if let Ok(v) = werte(&t) { let nv = bit_umschalten(v, n, b); d.text = format!("{nv}"); } }
            ZAkt::Kopieren(art) | ZAkt::Einsetzen(art) => {
                let Ok(v) = werte(&d.text) else { return Task::none() }; let text = match art { "hex" => hex(v), "bin" => binaer(v), _ => format!("{v}") };
                if matches!(a, ZAkt::Kopieren(_)) { return iced::clipboard::write(text) }
                if let Some(n) = self.name() { if let Some(e) = self.editoren.get_mut(&n) { e.insert(&text); self.zahlen = None; self.editor_fokus(); return Task::none() } } self.log.push(t::baum_keine_datei().into());
            }
        }
        Task::none()
    }
    pub(crate) fn zahlen_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let d = self.zahlen.as_ref()?; let z = |a: ZAkt| Msg::Zahl(a); let mono = vc_gui_ui::fonts::MONO;
        let wert = werte(&d.text);
        let mut c = column![text(vc_i18n::t!("zahlen-zahlenwerkzeuge")).size(16).color(pal.text), text(vc_i18n::t!("zahlen-zahl-oder-ausdruck-d020-0xff-1010")).size(11.5).color(pal.mute),
            text_input("$D020", &d.text).id("zahl").on_input(move |t| z(ZAkt::Text(t))).size(14).padding(7).font(mono)].spacing(8);
        match &wert {
            Err(e) => c = c.push(text(e.clone()).size(12).color(pal.err)),
            Ok(v) => {
                let v = *v; let b = d.breite; let zeile = |name: &str, w: String, art: Option<&'static str>| -> Element<Msg> { let mut r = row![text(name.to_string()).size(12).color(pal.mute).width(120), text(w).size(13).font(mono).color(pal.text).width(Length::Fill)].spacing(8).align_y(Alignment::Center); if let Some(a) = art { r = r.push(vc_gui_ui::muster::knopf(pal, &vc_i18n::t!("zahlen-kopieren"), &vc_i18n::t!("zahlen-in-die-zwischenablage"), Some(z(ZAkt::Kopieren(a))), false, false)).push(vc_gui_ui::muster::knopf(pal, &vc_i18n::t!("zahlen-einsetzen"), &vc_i18n::t!("zahlen-in-den-editor-an-der-schreibmarke"), Some(z(ZAkt::Einsetzen(a))), false, false)) } r.into() };
                c = c.push(zeile(&vc_i18n::t!("zahlen-dezimal"), format!("{v}"), Some("dez"))).push(zeile(&vc_i18n::t!("zahlen-hexadezimal"), hex(v), Some("hex"))).push(zeile(&vc_i18n::t!("zahlen-binaer"), binaer(v), Some("bin")))
                    .push(zeile(&vc_i18n::t!("zahlen-mit-vorzeichen-b-bit", b = &b), format!("{}", vorzeichen(v, b)), None)).push(zeile(&vc_i18n::t!("zahlen-lo-hi-byte"), format!("{} / {}", hex(v & 0xFF), hex((v >> 8) & 0xFF)), None)).push(zeile(&vc_i18n::t!("zahlen-zeichen-ascii"), zeichen(v).map_or("–".to_string(), |c| format!("„{c}“")), None));
                // Bit-Editor
                let mut bits = row![].spacing(3).align_y(Alignment::Center);
                for n in (0..b).rev() { let an = (v >> n) & 1 == 1; bits = bits.push(iced::widget::column![text(format!("{n}")).size(9.5).color(pal.mute), vc_gui_ui::muster::knopf(pal, if an { "1" } else { "0" }, &vc_i18n::t!("zahlen-bit-n-umschalten", n = &n), Some(z(ZAkt::Bit(n))), an, false)].spacing(2).align_x(Alignment::Center)); if n % 4 == 0 && n > 0 { bits = bits.push(space::horizontal().width(6)) } }
                c = c.push(text(vc_i18n::t!("zahlen-bit-editor")).size(12).color(pal.mute)).push(bits.wrap().vertical_spacing(4)).push(row![text(vc_i18n::t!("zahlen-breite")).size(12).color(pal.mute), vc_gui_ui::muster::knopf(pal, &vc_i18n::t!("zahlen-8-bit"), "", Some(z(ZAkt::Breite(8))), b == 8, false), vc_gui_ui::muster::knopf(pal, &vc_i18n::t!("zahlen-16-bit"), "", Some(z(ZAkt::Breite(16))), b == 16, false)].spacing(6).align_y(Alignment::Center));
            }
        }
        c = c.push(row![space::horizontal(), knopf(pal, vc_i18n::t!("zahlen-schliessen"), Some(z(ZAkt::Zu)), pal.text, false)]);
        let karte = container(c).width(560).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hintergrund = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() })).on_press(Msg::Zahl(ZAkt::Zu));
        Some(stack![hintergrund, container(karte).center(Length::Fill)].into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn schreibweisen_und_ausdruecke() {
        for (t, v) in [("$D020", 0xD020), ("0xFF", 255), ("%1010", 10), ("0b1010_0101", 0xA5), ("255", 255), ("-1", -1), ("$D000+$20", 0xD020), ("1<<5", 32), ("(1+2)*3", 9), ("$FF&%1111", 15), ("2+3*4", 14), ("$F0|$0F", 255), ("$ff ^ $0f", 0xF0), ("100/7", 14), ("~0", -1)] { assert_eq!(werte(t), Ok(v), "{t}"); }
        assert!(werte("").is_err() && werte("$").is_err() && werte("1+").is_err() && werte("1/0").is_err() && werte("12 ab").is_err() && werte("(1").is_err());
    }
    #[test] fn ausgabeformen_vorzeichen_und_bits() {
        assert_eq!((hex(5).as_str(), hex(0xD020).as_str(), hex(0x12345).as_str(), hex(-1).as_str()), ("$05", "$D020", "$12345", "-$1")); assert_eq!((binaer(5).as_str(), binaer(0x1FF).as_str()), ("%00000101", "%0000000111111111"));
        assert_eq!((vorzeichen(255, 8), vorzeichen(128, 8), vorzeichen(127, 8), vorzeichen(0xFFFF, 16), vorzeichen(-1, 8)), (-1, -128, 127, -1, -1));
        assert_eq!((zeichen(65), zeichen(10)), (Some('A'), None)); assert_eq!(bit_umschalten(0, 3, 8), 8); assert_eq!(bit_umschalten(8, 3, 8), 0); assert_eq!(bit_umschalten(0x1FF, 8, 8), 0x1FF, "Bit außerhalb der Breite bleibt");
    }
}
