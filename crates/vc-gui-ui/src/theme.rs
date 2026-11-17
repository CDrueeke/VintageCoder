// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: Farbrollen (Hell/Dunkel), wie in den Mockups.
use iced::Color;
/// Art eines Textstücks im Quelltext (für die Syntaxfarben)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Plain, Comment, Str, Num, Directive, Mnemonic, Label }

#[derive(Clone, Copy)]
pub struct Pal { pub dark: bool, pub hoch: bool, pub bg: Color, pub panel: Color, pub panel2: Color, pub line: Color, pub text: Color, pub mute: Color, pub accent: Color, pub ok: Color, pub warn: Color, pub err: Color, pub editor: Color, pub sel: Color, pub kw: Color, pub num: Color, pub string: Color, pub cmt: Color, pub lab: Color, pub dir: Color }
fn c(h: u32) -> Color { Color::from_rgb8((h >> 16) as u8, (h >> 8) as u8, h as u8) }
impl Pal {
    pub fn dark() -> Pal { Pal { dark: true, hoch: false, bg: c(0x14161a), panel: c(0x1b1e24), panel2: c(0x22262e), line: c(0x2e333d), text: c(0xe6e8ec), mute: c(0x8b93a1), accent: c(0x4f8cff), ok: c(0x3ecf8e), warn: c(0xf0a63a), err: c(0xff6b6b), editor: c(0x171a1f), sel: c(0x1b2840), kw: c(0xc792ea), num: c(0xf78c6c), string: c(0xc3e88d), cmt: c(0x7d8598), lab: c(0x82aaff), dir: c(0xffcb6b) } }
    pub fn light() -> Pal { Pal { dark: false, hoch: false, bg: c(0xf4f5f7), panel: c(0xffffff), panel2: c(0xeceef2), line: c(0xd5d9e0), text: c(0x1c2027), mute: c(0x5d6675), accent: c(0x1f5be0), ok: c(0x0f7655), warn: c(0x925300), err: c(0xc02424), editor: c(0xfbfbfc), sel: c(0xdfe9ff), kw: c(0x7e3fb8), num: c(0xb3470f), string: c(0x2f7a1f), cmt: c(0x65708a), lab: c(0x1f56c4), dir: c(0x825b00) } }
    /// Hoher Kontrast (dunkel): Text und Farbrollen mindestens 7:1 gegen jede Fläche, kräftige Rahmenlinien
    pub fn hoch() -> Pal { Pal { dark: true, hoch: true, bg: c(0x000000), panel: c(0x0a0a0a), panel2: c(0x1a1a1a), line: c(0x9a9a9a), text: c(0xffffff), mute: c(0xd0d0d0), accent: c(0x7dbaff), ok: c(0x5ff0a8), warn: c(0xffc247), err: c(0xff9a9a), editor: c(0x000000), sel: c(0x0f2552), kw: c(0xe0b3ff), num: c(0xffb08a), string: c(0xc8ff98), cmt: c(0xc0c8d8), lab: c(0xa8cdff), dir: c(0xffd780) } }
    /// Schriftfarbe auf Akzentflächen
    pub fn accent_text(&self) -> Color { if self.dark { c(0x0b0e14) } else { Color::WHITE } }
    /// Farbe der Art-Marke einer Datei im Dateibaum: nur Rollenfarben, die der Kontrasttest gegen alle Flächen prüft (Hell, Dunkel, Hoher Kontrast).
    /// Farbe ist nie der einzige Unterschied: die Marke steht als Text (ASM, BAS, SPR …) daneben. Gruppen teilen sich Farben (Code blau/grün, Klang orange,
    /// Bilder violett/Zeichen, Bildschirme/Kacheln/Paletten), Text und Binärdaten bleiben gedämpft. Legende in der Hilfe.
    pub fn art_farbe(&self, g: vc_ide_model::project::Group) -> Color {
        use vc_ide_model::project::Group::*;
        match g { Asm => self.accent, Basic => self.ok, Song | Play => self.warn, Sprite | Charset => self.lab, Screen | Tiles | Palette => self.dir, Note => self.text, Text | Binary | Other => self.mute }
    }
    pub fn kind(&self, k: Kind) -> Color { use Kind::*; match k { Plain => self.text, Comment => self.cmt, Str => self.string, Num => self.num, Directive => self.dir, Mnemonic => self.kw, Label => self.lab } }
}
