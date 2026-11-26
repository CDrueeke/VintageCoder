// SPDX-License-Identifier: GPL-3.0-or-later
//! Meldungen aus Schlüssel und benannten Parametern (i18n I2, K3).
//!
//! Eine [`Msg`] trägt keinen fertigen Text, sondern den Katalogschlüssel (`diag-<code>` oder `diag-<code>-<variante>`) und die Werte
//! als benannte Parameter. Der Text entsteht erst beim Formatieren ([`Msg::text_in`]) aus dem Katalog der gewünschten Sprache;
//! Werkzeuge (Sprachserver, Oberfläche, `vcc --json`) bekommen Schlüssel und Parameter maschinenlesbar.
//! Ein Parameter kann selbst eine Meldung sein (verschachtelt, z. B. „{ $inner } in '{ $text }'“).
use vc_i18n::{Arg, Lang};

/// Wert eines benannten Parameters
#[derive(Debug, Clone, PartialEq)]
pub enum MsgArg {
    Num(f64),
    Text(String),
    /// verschachtelte Meldung (wird beim Formatieren in derselben Sprache aufgelöst)
    Msg(Box<Msg>),
}

macro_rules! arg_num { ($($t:ty),*) => { $(impl From<$t> for MsgArg { fn from(v: $t) -> MsgArg { MsgArg::Num(v as f64) } })* } }
arg_num!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);
impl From<&str> for MsgArg { fn from(v: &str) -> MsgArg { MsgArg::Text(v.to_string()) } }
impl From<String> for MsgArg { fn from(v: String) -> MsgArg { MsgArg::Text(v) } }
impl From<char> for MsgArg { fn from(v: char) -> MsgArg { MsgArg::Text(v.to_string()) } }
impl From<&std::io::Error> for MsgArg { fn from(v: &std::io::Error) -> MsgArg { MsgArg::Text(v.to_string()) } }
impl From<std::path::Display<'_>> for MsgArg { fn from(v: std::path::Display<'_>) -> MsgArg { MsgArg::Text(v.to_string()) } }
impl From<Msg> for MsgArg { fn from(v: Msg) -> MsgArg { MsgArg::Msg(Box::new(v)) } }
impl<T: Clone + Into<MsgArg>> From<&T> for MsgArg { fn from(v: &T) -> MsgArg { v.clone().into() } }

impl MsgArg {
    /// Text des Parameters (Zahlen ohne Nachkommastellen, wenn ganz)
    pub fn as_text(&self) -> String {
        match self {
            MsgArg::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => format!("{}", *n as i64),
            MsgArg::Num(n) => format!("{n}"),
            MsgArg::Text(t) => t.clone(),
            MsgArg::Msg(m) => m.text(),
        }
    }
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            MsgArg::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => serde_json::json!(*n as i64),
            MsgArg::Num(n) => serde_json::json!(n),
            MsgArg::Text(t) => serde_json::json!(t),
            MsgArg::Msg(m) => m.to_json(),
        }
    }
}

/// Meldung: Katalogschlüssel und benannte Parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Msg {
    pub key: &'static str,
    pub args: Vec<(&'static str, MsgArg)>,
}

impl Msg {
    pub fn new(key: &'static str) -> Msg { Msg { key, args: Vec::new() } }
    pub fn with(mut self, name: &'static str, v: impl Into<MsgArg>) -> Msg { self.args.push((name, v.into())); self }
    /// Text in der aktuellen Sprache
    pub fn text(&self) -> String { self.text_in(vc_i18n::language()) }
    /// Text in `lang` (verschachtelte Meldungen in derselben Sprache)
    pub fn text_in(&self, lang: Lang) -> String {
        let args: Vec<(&str, Arg)> = self.args.iter().map(|(k, v)| (*k, match v {
            MsgArg::Num(n) => Arg::Num(*n),
            MsgArg::Text(t) => Arg::Text(t.clone()),
            MsgArg::Msg(m) => Arg::Text(m.text_in(lang)),
        })).collect();
        vc_i18n::translate_in(lang, self.key, &args)
    }
    /// Wert des Parameters `name`, wenn vorhanden
    pub fn arg(&self, name: &str) -> Option<&MsgArg> { self.args.iter().find(|(k, _)| *k == name).map(|(_, v)| v) }
    /// Text des Parameters `name` (leer, wenn er fehlt)
    pub fn arg_text(&self, name: &str) -> String { self.arg(name).map(MsgArg::as_text).unwrap_or_default() }
    /// `{ "key": …, "args": { name: wert } }`; verschachtelte Meldungen erscheinen als Objekt derselben Form
    pub fn to_json(&self) -> serde_json::Value {
        let mut a = serde_json::Map::new();
        for (k, v) in &self.args { a.insert((*k).to_string(), v.to_json()); }
        serde_json::json!({ "key": self.key, "args": a })
    }
    /// Meldung ohne Katalogvorlage (nur für Aufrufer außerhalb der übersetzten Crates; `tools/check_i18n.py` zählt sie)
    pub fn raw(text: impl Into<String>) -> Msg { Msg::new("diag-text").with("text", text.into()) }
}

/// `msg!("schlüssel")` oder `msg!("schlüssel", name = wert, n = 3)`: [`Msg`] mit benannten Parametern
#[macro_export]
macro_rules! msg {
    ($key:expr) => { $crate::Msg::new($key) };
    ($key:expr, $($name:ident = $val:expr),+ $(,)?) => { $crate::Msg { key: $key, args: vec![$((stringify!($name), $crate::MsgArg::from($val))),+] } };
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Diagnostic, Range};

    #[test]
    fn text_folgt_der_sprache_und_json_hat_key_und_args() {
        let m = msg!("core-chars-bad-code", inner = "$zz");
        assert_eq!(m.text_in(Lang::De), "ungültiger Code {$zz}");
        assert_eq!(m.text_in(Lang::En), "invalid code {$zz}");
        let d = Diagnostic::error("E999", "a.asm", 7, m).with_range(Range { line: 7, col: 3, end_line: 7, end_col: 9 });
        assert_eq!((d.code, d.key, d.line), ("E999", "core-chars-bad-code", 7));
        assert_eq!(d.arg_text("inner"), "$zz");
        let j = d.to_json(Lang::En);
        assert_eq!(j["code"], "E999");
        assert_eq!(j["key"], "core-chars-bad-code");
        assert_eq!(j["args"]["inner"], "$zz");
        assert_eq!(j["range"]["col"], 3);
        assert_eq!(j["message"], "invalid code {$zz}");
    }

    #[test]
    fn verschachtelte_meldung_wird_in_derselben_sprache_aufgeloest() {
        let inner = msg!("core-textfile-binary");
        let m = Msg::new("diag-text").with("text", inner.clone());
        assert_eq!(m.text_in(Lang::De), "Binärdatei");
        assert_eq!(m.text_in(Lang::En), "Binary file");
        assert_eq!(m.to_json()["args"]["text"]["key"], "core-textfile-binary");
    }
}
/// Fertiger Text in der aktuellen Sprache (für Aufrufer, die Fehler als `String` weiterreichen)
impl From<Msg> for String { fn from(m: Msg) -> String { m.text() } }
impl std::fmt::Display for Msg { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.text()) } }
