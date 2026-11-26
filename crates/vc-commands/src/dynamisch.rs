// SPDX-License-Identifier: GPL-3.0-or-later
//! Dynamische Befehle aus der Registry (Erweiterungen): neben der festen Tabelle [`crate::BEFEHLE`]. Kennungen haben den Namensraum `ext:<paket>:<befehl>`;
//! die Titel kommen mit dem Befehl (je Sprache, Rückfall Englisch). Menü, Befehlspalette und Kürzel lesen die vereinigte Tabelle ([`zeilen`]).
//! Kürzel, die schon belegt sind (fest oder von einem anderen dynamischen Befehl), werden **nicht überschrieben**: der Befehl bleibt ohne Kürzel, die Meldung steht in der Rückgabe.
//! Das Untermenü „Projekt ▸ Erweiterungen“ gibt es nur, solange es dynamische Befehle gibt.
use crate::{Braucht, BEFEHLE, MENUES};
use std::sync::RwLock;

/// Name des Untermenüs für Erweiterungsbefehle
pub const UNTERMENUE_ERWEITERUNGEN: &str = "Erweiterungen";

/// Ein Befehl einer Erweiterung
#[derive(Clone, Debug, PartialEq)]
pub struct Befehl {
    /// `ext:<paket>:<befehl>`
    pub id: String,
    /// Titel je Sprache (`de`, `en`)
    pub titel: Vec<(String, String)>,
    pub menue: String,
    pub untermenue: String,
    pub tasten: String,
    pub braucht: Braucht,
    /// Kennung des Pakets (Herkunft)
    pub herkunft: String,
    /// Editor-Kennung der Release-Auswahl (`sprite`, `tracker` …): ohne diesen Editor im Release entfällt der Befehl
    pub feature: Option<String>,
}
impl Befehl {
    /// Titel in `lang` (`de`, `en`); sonst Englisch, sonst der erste, sonst die Kennung
    pub fn titel_in(&self, lang: &str) -> String {
        let find = |l: &str| self.titel.iter().find(|(k, _)| k == l).map(|(_, t)| t.clone());
        find(lang).or_else(|| find("en")).or_else(|| self.titel.first().map(|(_, t)| t.clone())).unwrap_or_else(|| self.id.clone())
    }
}

/// Eine Zeile der vereinigten Tabelle (fest oder dynamisch)
#[derive(Clone, Debug, PartialEq)]
pub struct Zeile {
    pub id: String,
    pub name: String,
    pub menue: String,
    pub unter: String,
    pub tasten: String,
    pub braucht: Braucht,
    /// `None` bei festen Befehlen
    pub herkunft: Option<String>,
    pub feature: Option<String>,
    pub dynamisch: bool,
}

static DYN: RwLock<Vec<Befehl>> = RwLock::new(Vec::new());

fn gueltige_id(id: &str) -> bool {
    let mut t = id.split(':');
    t.next() == Some("ext") && t.clone().count() == 2 && t.all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.')))
}
fn norm(t: &str) -> String { t.to_ascii_lowercase().replace(' ', "") }

/// Setzt die dynamischen Befehle (ersetzt die bisherigen). Rückgabe: Meldungen zu abgelehnten Befehlen und entfernten Kürzeln (leer = alles übernommen)
pub fn setze(neu: Vec<Befehl>) -> Vec<String> {
    let mut meldungen = vec![]; let mut ok: Vec<Befehl> = vec![];
    let feste_tasten: Vec<String> = BEFEHLE.iter().filter(|i| !i.tasten.is_empty()).map(|i| norm(i.tasten)).collect();
    for mut b in neu {
        if !gueltige_id(&b.id) { meldungen.push(format!("{}: ungültige Befehlskennung (erwartet ext:<paket>:<befehl>)", b.id)); continue; }
        if ok.iter().any(|x| x.id == b.id) { meldungen.push(format!("{}: Kennung doppelt", b.id)); continue; }
        if !MENUES.contains(&b.menue.as_str()) { b.menue = "Projekt".to_string(); }
        if b.untermenue.is_empty() { b.untermenue = UNTERMENUE_ERWEITERUNGEN.to_string(); }
        if !b.tasten.is_empty() {
            let n = norm(&b.tasten);
            if feste_tasten.contains(&n) || ok.iter().any(|x| !x.tasten.is_empty() && norm(&x.tasten) == n) { meldungen.push(format!("{}: Kürzel {} ist schon belegt und wird nicht gesetzt", b.id, b.tasten)); b.tasten.clear(); }
        }
        ok.push(b);
    }
    *DYN.write().unwrap() = ok;
    meldungen
}
/// Die dynamischen Befehle (Kopie)
pub fn dynamische() -> Vec<Befehl> { DYN.read().unwrap().clone() }
pub fn hat_dynamische() -> bool { !DYN.read().unwrap().is_empty() }

/// Vereinigte Tabelle: zuerst die festen Befehle in Tabellenreihenfolge, danach die dynamischen. `lang` für die Titel dynamischer Befehle.
pub fn zeilen(lang: &str) -> Vec<Zeile> {
    let mut v: Vec<Zeile> = BEFEHLE.iter().map(|i| Zeile { id: i.id.to_string(), name: i.name(), menue: i.menue.to_string(), unter: crate::untermenue(i.cmd).to_string(), tasten: i.tasten.to_string(), braucht: i.braucht, herkunft: None, feature: crate::feature_von(i.cmd).map(String::from), dynamisch: false }).collect();
    v.extend(DYN.read().unwrap().iter().map(|b| Zeile { id: b.id.clone(), name: b.titel_in(lang), menue: b.menue.clone(), unter: b.untermenue.clone(), tasten: b.tasten.clone(), braucht: b.braucht, herkunft: Some(b.herkunft.clone()), feature: b.feature.clone(), dynamisch: true }));
    v
}
/// Untermenüs eines Menüs: die festen und, falls dynamische Befehle dort stehen, deren Untermenüs (nach den festen)
pub fn untermenues_alle(menue: &str) -> Vec<String> {
    let mut v: Vec<String> = crate::untermenues(menue).iter().map(|s| s.to_string()).collect();
    for b in DYN.read().unwrap().iter() { if b.menue == menue && !b.untermenue.is_empty() && !v.contains(&b.untermenue) { v.push(b.untermenue.clone()); } }
    v
}
/// Titel eines Untermenüs (die festen aus dem Katalog; „Erweiterungen“ ebenfalls dort; unbekannte so, wie sie dastehen)
pub fn untermenue_titel_alle(u: &str) -> String {
    if u == UNTERMENUE_ERWEITERUNGEN { return vc_i18n::t!("submenu-erweiterungen").into(); }
    let key = format!("submenu-{}", crate::schluessel_teil_pub(u));
    if vc_i18n::has_key(&key) { vc_i18n::translate(&key, &[]) } else { u.to_string() }
}

/// Name im Menü: in einem Untermenü ohne dessen Namen als Vorspann (wie [`crate::menuename`] für die festen Befehle)
pub fn menuename(z: &Zeile) -> String {
    if z.unter.is_empty() { return z.name.clone(); }
    z.name.strip_prefix(&format!("{}: ", untermenue_titel_alle(&z.unter))).map(str::to_string).unwrap_or_else(|| z.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn b(id: &str, tasten: &str) -> Befehl { Befehl { id: id.into(), titel: vec![("de".into(), "Hallo".into()), ("en".into(), "Hello".into())], menue: "Projekt".into(), untermenue: String::new(), tasten: tasten.into(), braucht: Braucht::Nichts, herkunft: "autor.paket".into(), feature: None } }
    #[test]
    fn befehle_kennungen_kuerzel_und_untermenue() {
        let _g = crate::test_sperre();
        assert!(!hat_dynamische() && untermenues_alle("Projekt") == crate::untermenues("Projekt").iter().map(|s| s.to_string()).collect::<Vec<_>>(), "ohne Erweiterungen bleibt das Menü wie es ist");
        let m = setze(vec![b("ext:autor.paket:hallo", ""), b("hallo", ""), b("ext:autor.paket:hallo", ""), b("ext:autor.paket:zwei", "Cmd+S"), b("ext:autor.paket:drei", "Cmd+Alt+Umschalt+Y"), b("ext:autor.paket:vier", "cmd + alt + umschalt + y"), b("ext:a:b:c", "")]);
        assert_eq!(m.len(), 5, "{m:?}");
        assert!(m.iter().any(|x| x.contains("ungültig") && x.starts_with("hallo")) && m.iter().any(|x| x.contains("doppelt")) && m.iter().filter(|x| x.contains("Kürzel")).count() == 2);
        let z = zeilen("de"); let e: Vec<&Zeile> = z.iter().filter(|x| x.dynamisch).collect();
        assert_eq!(e.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["ext:autor.paket:hallo", "ext:autor.paket:zwei", "ext:autor.paket:drei", "ext:autor.paket:vier"]);
        assert!(e[1].tasten.is_empty() && e[2].tasten == "Cmd+Alt+Umschalt+Y" && e[3].tasten.is_empty(), "belegte Kürzel werden nicht überschrieben");
        assert!(e.iter().all(|x| x.menue == "Projekt" && x.unter == UNTERMENUE_ERWEITERUNGEN && x.herkunft.as_deref() == Some("autor.paket")));
        assert_eq!(untermenues_alle("Projekt").last().map(String::as_str), Some(UNTERMENUE_ERWEITERUNGEN));
        assert_eq!(e[0].name, "Hallo"); assert_eq!(zeilen("en").iter().find(|x| x.dynamisch).unwrap().name, "Hello"); assert_eq!(zeilen("xx").iter().find(|x| x.dynamisch).unwrap().name, "Hello", "Rückfall Englisch");
        assert_eq!(z.iter().filter(|x| !x.dynamisch).count(), BEFEHLE.len());
        setze(vec![]); assert!(!hat_dynamische());
    }
}
