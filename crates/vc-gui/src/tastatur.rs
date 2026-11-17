// SPDX-License-Identifier: GPL-3.0-or-later
//! Tastaturlayout für die Klaviertasten des Trackers. **Belegung nach der Position der Taste** (`physical_key`): die untere Reihe Z S X D C V G B H N J M und die obere
//! Reihe Q 2 W 3 E R 5 T 6 Y 7 U I sind Positionen wie auf einer US-QWERTY-Tastatur und liegen auf jedem Layout am selben Ort. Nur die **Anzeige** (Hinweise, Beschriftung
//! der Klaviatur-Karte) richtet sich nach dem Layout: auf QWERTZ steht an der Stelle des Z ein Y und umgekehrt, auf AZERTY sitzen A/Q, Z/W und M anders.
//! Das Layout kommt aus der Einstellung oder, bei „Automatisch“, aus dem Betriebssystem und aus dem, was die Tasten selbst verraten (logische gegen physische Taste).
use crate::einstellungen::Tastaturlayout;
use iced::keyboard::{key::{Code, Physical}, Key};

/// Untere Reihe (Halbtöne 0–11) und obere Reihe (12–24) als QWERTY-Positionen
pub const UNTEN: [char; 12] = ['z', 's', 'x', 'd', 'c', 'v', 'g', 'b', 'h', 'n', 'j', 'm'];
pub const OBEN: [char; 13] = ['q', '2', 'w', '3', 'e', 'r', '5', 't', '6', 'y', '7', 'u', 'i'];

/// QWERTY-Zeichen der physischen Taste (nur Buchstaben und Ziffern; alles andere bleibt logisch)
pub fn physisch_zeichen(p: &Physical) -> Option<char> {
    let Physical::Code(c) = p else { return None };
    Some(match c {
        Code::KeyA => 'a', Code::KeyB => 'b', Code::KeyC => 'c', Code::KeyD => 'd', Code::KeyE => 'e', Code::KeyF => 'f', Code::KeyG => 'g', Code::KeyH => 'h', Code::KeyI => 'i', Code::KeyJ => 'j', Code::KeyK => 'k', Code::KeyL => 'l', Code::KeyM => 'm',
        Code::KeyN => 'n', Code::KeyO => 'o', Code::KeyP => 'p', Code::KeyQ => 'q', Code::KeyR => 'r', Code::KeyS => 's', Code::KeyT => 't', Code::KeyU => 'u', Code::KeyV => 'v', Code::KeyW => 'w', Code::KeyX => 'x', Code::KeyY => 'y', Code::KeyZ => 'z',
        Code::Digit0 => '0', Code::Digit1 => '1', Code::Digit2 => '2', Code::Digit3 => '3', Code::Digit4 => '4', Code::Digit5 => '5', Code::Digit6 => '6', Code::Digit7 => '7', Code::Digit8 => '8', Code::Digit9 => '9',
        _ => return None,
    })
}

/// Was die Tasten über das Layout verraten: logische gegen physische Taste (nur eindeutige Paare)
pub fn lernen(k: &Key, p: &Physical) -> Option<Tastaturlayout> {
    let (Key::Character(c), Some(ph)) = (k, physisch_zeichen(p)) else { return None };
    let l = c.chars().next()?.to_ascii_lowercase();
    match (ph, l) {
        ('z', 'y') | ('y', 'z') => Some(Tastaturlayout::Qwertz),
        ('q', 'a') | ('a', 'q') | ('w', 'z') | ('z', 'w') => Some(Tastaturlayout::Azerty),
        ('z', 'z') | ('y', 'y') => Some(Tastaturlayout::Qwerty),
        _ => None,
    }
}

/// Aufgelöstes Layout: Einstellung, sonst Gelerntes, sonst Betriebssystem, sonst QWERTY
pub fn aufloesen(einst: Tastaturlayout, gelernt: Option<Tastaturlayout>, system: Option<Tastaturlayout>) -> Tastaturlayout {
    match einst { Tastaturlayout::Auto => gelernt.or(system).unwrap_or(Tastaturlayout::Qwerty), l => l }
}

/// Beschriftung der Taste an der QWERTY-Position `c` auf dem Layout `l`
pub fn beschriftung(l: Tastaturlayout, c: char) -> String {
    let b = match l {
        Tastaturlayout::Qwertz => match c { 'z' => 'y', 'y' => 'z', x => x },
        Tastaturlayout::Azerty => match c { 'q' => 'a', 'a' => 'q', 'w' => 'z', 'z' => 'w', 'm' => ',', x => x },
        _ => c,
    };
    b.to_uppercase().to_string()
}
/// „Z S X D …“ der unteren Reihe in der Beschriftung des Layouts
pub fn hinweis_unten(l: Tastaturlayout) -> String { UNTEN.iter().map(|c| beschriftung(l, *c)).collect::<Vec<_>>().join(" ") }
/// „Q 2 W 3 …“ der oberen Reihe
pub fn hinweis_oben(l: Tastaturlayout) -> String { OBEN.iter().map(|c| beschriftung(l, *c)).collect::<Vec<_>>().join(" ") }
/// Beschriftung der Computertaste für den Halbton `h` über dem C (0–24)
pub fn fuer_halbton(l: Tastaturlayout, h: usize) -> Option<String> { if h < 12 { Some(beschriftung(l, UNTEN[h])) } else { OBEN.get(h - 12).map(|c| beschriftung(l, *c)) } }
pub fn name(l: Tastaturlayout) -> &'static str { match l { Tastaturlayout::Auto => "Automatisch", Tastaturlayout::Qwertz => "QWERTZ", Tastaturlayout::Qwerty => "QWERTY", Tastaturlayout::Azerty => "AZERTY" } }

/// Layout des Betriebssystems, soweit abfragbar (die Abfrage selbst steht in `aussen`)
pub fn system_layout() -> Option<Tastaturlayout> { crate::aussen::tastaturlayout_text().and_then(|r| aus_text(&r)) }
/// Layout aus dem Text der Systemabfrage (klein geschrieben)
pub fn aus_text(roh: &str) -> Option<Tastaturlayout> {
    let roh = roh.to_lowercase();
    if ["german", "swiss", "austrian", "layout:     de", "layout:     ch", "layout:     at", "czech", "slovak", "hungarian", "croatian", "slovenian"].iter().any(|k| roh.contains(k)) { Some(Tastaturlayout::Qwertz) }
    else if ["french", "belgian", "layout:     fr", "layout:     be"].iter().any(|k| roh.contains(k)) { Some(Tastaturlayout::Azerty) }
    else if roh.trim().is_empty() { None } else { Some(Tastaturlayout::Qwerty) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn beschriftung_je_layout() {
        assert_eq!(hinweis_unten(Tastaturlayout::Qwerty), "Z S X D C V G B H N J M"); assert_eq!(hinweis_oben(Tastaturlayout::Qwerty), "Q 2 W 3 E R 5 T 6 Y 7 U I");
        assert_eq!(hinweis_unten(Tastaturlayout::Qwertz), "Y S X D C V G B H N J M", "QWERTZ: an der Stelle des Z steht ein Y"); assert_eq!(hinweis_oben(Tastaturlayout::Qwertz), "Q 2 W 3 E R 5 T 6 Z 7 U I");
        assert_eq!(hinweis_unten(Tastaturlayout::Azerty), "W S X D C V G B H N J ,"); assert_eq!(hinweis_oben(Tastaturlayout::Azerty), "A 2 Z 3 E R 5 T 6 Y 7 U I");
        assert_eq!((fuer_halbton(Tastaturlayout::Qwertz, 0).as_deref(), fuer_halbton(Tastaturlayout::Qwertz, 21).as_deref(), fuer_halbton(Tastaturlayout::Qwertz, 25)), (Some("Y"), Some("Z"), None));
    }
    #[test] fn systemtext_wird_zum_layout() {
        assert_eq!(aus_text("com.apple.keylayout.German"), Some(Tastaturlayout::Qwertz)); assert_eq!(aus_text("com.apple.keylayout.French"), Some(Tastaturlayout::Azerty)); assert_eq!(aus_text("com.apple.keylayout.US"), Some(Tastaturlayout::Qwerty)); assert_eq!(aus_text("  "), None);
    }
    #[test] fn layout_wird_aus_den_tasten_gelernt_und_aufgeloest() {
        let ev = |l: &str, p: Code| (Key::Character(l.into()), Physical::Code(p));
        let (k, p) = ev("y", Code::KeyZ); assert_eq!(lernen(&k, &p), Some(Tastaturlayout::Qwertz)); let (k, p) = ev("z", Code::KeyY); assert_eq!(lernen(&k, &p), Some(Tastaturlayout::Qwertz));
        let (k, p) = ev("a", Code::KeyQ); assert_eq!(lernen(&k, &p), Some(Tastaturlayout::Azerty)); let (k, p) = ev("z", Code::KeyZ); assert_eq!(lernen(&k, &p), Some(Tastaturlayout::Qwerty)); let (k, p) = ev("d", Code::KeyD); assert_eq!(lernen(&k, &p), None);
        assert_eq!(aufloesen(Tastaturlayout::Auto, None, None), Tastaturlayout::Qwerty); assert_eq!(aufloesen(Tastaturlayout::Auto, Some(Tastaturlayout::Qwertz), Some(Tastaturlayout::Qwerty)), Tastaturlayout::Qwertz);
        assert_eq!(aufloesen(Tastaturlayout::Azerty, Some(Tastaturlayout::Qwertz), None), Tastaturlayout::Azerty, "feste Einstellung gewinnt");
        assert_eq!(physisch_zeichen(&Physical::Code(Code::Digit2)), Some('2')); assert_eq!(physisch_zeichen(&Physical::Code(Code::Comma)), None);
    }
}
