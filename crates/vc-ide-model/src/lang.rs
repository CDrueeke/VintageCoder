// SPDX-License-Identifier: GPL-3.0-or-later
//! Dateiart eines Textes (aus der Endung): bestimmt Färbung, Sprachdienste und Zeichen-Einfügen.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang { Asm, Basic, Plain, Markdown }

impl Lang {
    pub fn of(path: &str) -> Lang {
        let p = path.to_ascii_lowercase();
        if p.ends_with(".asm") || p.ends_with(".inc") || p.ends_with(".s") { Lang::Asm } else if p.ends_with(".sbas") || p.ends_with(".bas") || p.ends_with(".lst") || p.ends_with(".tur") { Lang::Basic } else if p.ends_with(".md") || p.ends_with(".markdown") { Lang::Markdown } else { Lang::Plain }
    }
    pub fn name(self) -> &'static str { match self { Lang::Asm => "Assembler", Lang::Basic => "BASIC", Lang::Plain => "Text", Lang::Markdown => "Markdown" } }
    /// Textdateien ohne Programmiersprache (Text und Markdown): keine Übersetzung, keine Sprachdienste, kein Zeichen-Einfügen
    pub fn is_text(self) -> bool { matches!(self, Lang::Plain | Lang::Markdown) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn endungen() { assert!(Lang::of("A.ASM") == Lang::Asm); assert!(Lang::of("x.bas") == Lang::Basic); assert!(Lang::of("r.md") == Lang::Markdown); assert!(Lang::of("x.txt") == Lang::Plain); assert!(Lang::Markdown.is_text() && !Lang::Asm.is_text()); }
}
