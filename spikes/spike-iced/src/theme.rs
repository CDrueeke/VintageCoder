// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: Farbrollen (Hell/Dunkel), wie in den Mockups.
use iced::Color;
#[derive(Clone, Copy)]
pub struct Pal { pub dark: bool, pub bg: Color, pub panel: Color, pub panel2: Color, pub line: Color, pub text: Color, pub mute: Color, pub accent: Color, pub ok: Color, pub warn: Color, pub err: Color, pub editor: Color, pub sel: Color, pub kw: Color, pub num: Color, pub string: Color, pub cmt: Color, pub lab: Color, pub dir: Color }
fn c(h: u32) -> Color { Color::from_rgb8((h >> 16) as u8, (h >> 8) as u8, h as u8) }
impl Pal {
    pub fn dark() -> Pal { Pal { dark: true, bg: c(0x14161a), panel: c(0x1b1e24), panel2: c(0x22262e), line: c(0x2e333d), text: c(0xe6e8ec), mute: c(0x8b93a1), accent: c(0x4f8cff), ok: c(0x3ecf8e), warn: c(0xf0a63a), err: c(0xff6b6b), editor: c(0x171a1f), sel: c(0x2a3a5c), kw: c(0xc792ea), num: c(0xf78c6c), string: c(0xc3e88d), cmt: c(0x6b7385), lab: c(0x82aaff), dir: c(0xffcb6b) } }
    pub fn light() -> Pal { Pal { dark: false, bg: c(0xf4f5f7), panel: c(0xffffff), panel2: c(0xeceef2), line: c(0xd5d9e0), text: c(0x1c2027), mute: c(0x5d6675), accent: c(0x2563eb), ok: c(0x12805c), warn: c(0xa15c00), err: c(0xc92a2a), editor: c(0xfbfbfc), sel: c(0xcfe0ff), kw: c(0x7e3fb8), num: c(0xb3470f), string: c(0x2f7a1f), cmt: c(0x7a8396), lab: c(0x1f56c4), dir: c(0x8a6100) } }
    pub fn kind(&self, k: crate::highlight::Kind) -> Color { use crate::highlight::Kind::*; match k { Plain => self.text, Comment => self.cmt, Str => self.string, Num => self.num, Directive => self.dir, Mnemonic => self.kw, Label => self.lab } }
}
