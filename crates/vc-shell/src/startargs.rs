// SPDX-License-Identifier: GPL-3.0-or-later
//! Start mit Pfaden: `VintageCoder <pfad> […] [--open <datei>] [--view <ansicht>]` und „Öffnen mit“ des Betriebssystems (macOS: `open -a VintageCoder <pfad>`, Doppelklick auf eine Datei).
//! Die Pfade warten hier, bis die Oberfläche sie abholt (`start_args`); kommt später etwas dazu (zweiter Aufruf unter macOS), meldet das Ereignis `app:oeffnen`.
use serde_json::{json, Value};
use std::sync::Mutex;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Auftrag { pub pfade: Vec<String>, /** Datei im angegebenen Projekt, die danach geöffnet wird */ pub open: Option<String>, /** Ansicht, die danach gezeigt wird (`sprites`, `disk` …) */ pub view: Option<String> }

static OFFEN: Mutex<Auftrag> = Mutex::new(Auftrag { pfade: vec![], open: None, view: None });

/// Auftrag aus den Argumenten (ohne Programmnamen). Unbekannte `--…`-Schalter werden ignoriert (Betriebssysteme hängen eigene an, z. B. `-psn_…`).
pub fn aus_argumenten<I: IntoIterator<Item = String>>(args: I) -> Auftrag {
    let mut a = Auftrag::default(); let mut it = args.into_iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--open" => { a.open = it.next(); }
            "--view" => { a.view = it.next(); }
            s if s.starts_with("--open=") => { a.open = Some(s["--open=".len()..].to_string()); }
            s if s.starts_with("--view=") => { a.view = Some(s["--view=".len()..].to_string()); }
            s if s.starts_with('-') => {}
            _ => a.pfade.push(x),
        }
    }
    a
}

/// Auftrag vormerken (zu schon wartenden Pfaden hinzufügen)
pub fn vormerken(a: Auftrag) {
    let mut o = OFFEN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    o.pfade.extend(a.pfade); if a.open.is_some() { o.open = a.open; } if a.view.is_some() { o.view = a.view; }
}
/// Wartenden Auftrag abholen (danach leer)
pub fn abholen() -> Value {
    let a = std::mem::take(&mut *OFFEN.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
    json!({ "paths": a.pfade, "open": a.open, "view": a.view })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s(v: &[&str]) -> Vec<String> { v.iter().map(|x| x.to_string()).collect() }
    #[test]
    fn argumente_pfade_und_schalter() {
        let a = aus_argumenten(s(&["/p/projekt", "/d/x.d64", "--open", "src/a.asm", "--view=sprites", "-psn_0_123", "--unbekannt"]));
        assert_eq!(a, Auftrag { pfade: s(&["/p/projekt", "/d/x.d64"]), open: Some("src/a.asm".into()), view: Some("sprites".into()) });
        assert_eq!(aus_argumenten(s(&[])), Auftrag::default());
    }
    #[test]
    fn vormerken_und_abholen() {
        vormerken(Auftrag { pfade: s(&["/a"]), open: None, view: None }); vormerken(Auftrag { pfade: s(&["/b"]), open: Some("x".into()), view: None });
        let v = abholen(); assert_eq!(v["paths"], json!(["/a", "/b"])); assert_eq!(v["open"], "x");
        assert_eq!(abholen()["paths"], json!([]), "nach dem Abholen leer");
    }
}
