// SPDX-License-Identifier: GPL-3.0-or-later
//! Layoutmodell der Arbeitsfläche (GUI-frei): welche Bereiche normal, minimiert oder maximiert sind.
//! Regeln (Konzept Abschnitt 5.1): jeder Bereich lässt sich zu einem Streifen/einer Leiste minimieren (außer dem Dokumentenfeld),
//! einer füllt die Fläche (maximiert), die übrigen sind dann eingeklappt; Wiederherstellen stellt das vorige Layout her.
//! Gemerkt wird nur Normal/Minimiert; „maximiert“ nie.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bereich { Seitenleiste, Dokument, Schublade, Panel }
impl Bereich {
    pub const ALLE: [Bereich; 4] = [Bereich::Seitenleiste, Bereich::Dokument, Bereich::Schublade, Bereich::Panel];
    fn index(self) -> usize { self as usize }
}

/// Was die Oberfläche von einem Bereich zeigt
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anzeige { Normal, Minimiert, Maximiert }

/// Gemerkter Teil (Einstellungen): minimiert ja/nein je Bereich
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Gemerkt { pub seitenleiste_minimiert: bool, pub schublade_minimiert: bool, pub panel_minimiert: bool }
impl Default for Gemerkt { fn default() -> Self { Gemerkt { seitenleiste_minimiert: false, schublade_minimiert: false, panel_minimiert: true } } }

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    min: [bool; 4],
    max: Option<Bereich>,
    /// Layout vor einer Debug-Sitzung
    vor_debug: Option<([bool; 4], Option<Bereich>)>,
    /// Zen-Modus: das Layout davor (Seitenleiste, Schublade, Panel sind eingeklappt, die Studios zeigen nur die Fläche); `None` = aus
    zen: Option<([bool; 4], Option<Bereich>)>,
}

impl Default for Layout { fn default() -> Self { Layout::vorgabe() } }

impl Layout {
    /// Vorgabe: Spalte und Schublade normal, Panel als Streifen (Entscheidung 8)
    pub fn vorgabe() -> Layout { Layout::aus_gemerkt(Gemerkt::default()) }
    pub fn aus_gemerkt(g: Gemerkt) -> Layout {
        let mut min = [false; 4]; min[Bereich::Seitenleiste.index()] = g.seitenleiste_minimiert; min[Bereich::Schublade.index()] = g.schublade_minimiert; min[Bereich::Panel.index()] = g.panel_minimiert;
        Layout { min, max: None, vor_debug: None, zen: None }
    }
    /// Gemerkt wird im Zen-Modus das Layout davor (der Zen-Zustand selbst nie)
    pub fn gemerkt(&self) -> Gemerkt { let m = self.zen.map_or(self.min, |z| z.0); Gemerkt { seitenleiste_minimiert: m[0], schublade_minimiert: m[2], panel_minimiert: m[3] } }

    /// Der maximierte Bereich, falls einer
    pub fn maximiert(&self) -> Option<Bereich> { self.max }

    /// Was der Bereich jetzt zeigt: bei einem Maximierten sind alle anderen eingeklappt (das Dokumentenfeld als Reiterleiste)
    pub fn anzeige(&self, b: Bereich) -> Anzeige {
        match self.max { Some(m) if m == b => Anzeige::Maximiert, Some(_) => Anzeige::Minimiert, None => if self.min[b.index()] { Anzeige::Minimiert } else { Anzeige::Normal } }
    }
    /// Zeigt der Bereich seinen Inhalt (normal oder maximiert)?
    pub fn offen(&self, b: Bereich) -> bool { self.anzeige(b) != Anzeige::Minimiert }
    pub fn kann_minimieren(&self, b: Bereich) -> bool { b != Bereich::Dokument }

    /// Bereich zum Streifen/zur Leiste einklappen; ein maximierter Bereich wird dabei wiederhergestellt und danach minimiert. Das Dokumentenfeld bleibt (liefert `false`).
    pub fn minimieren(&mut self, b: Bereich) -> bool {
        if !self.kann_minimieren(b) { return false }
        if self.max == Some(b) { self.max = None }
        self.min[b.index()] = true; true
    }
    /// Bereich aufklappen (Normal); war ein anderer Bereich maximiert, endet das
    pub fn aufklappen(&mut self, b: Bereich) {
        if self.max.is_some_and(|m| m != b) { self.max = None }
        if self.max == Some(b) { self.max = None }
        self.min[b.index()] = false;
    }
    /// Minimiert ↔ Normal (Cmd/Strg+B, J, Alt+B)
    pub fn umschalten(&mut self, b: Bereich) {
        if self.anzeige(b) == Anzeige::Normal { self.minimieren(b); } else { self.aufklappen(b) }
    }
    /// Bereich füllt die Arbeitsfläche; die gemerkten Zustände bleiben unberührt (Wiederherstellen stellt sie her)
    pub fn maximieren(&mut self, b: Bereich) { self.max = Some(b) }
    pub fn wiederherstellen(&mut self) { self.max = None }
    /// Maximiert ↔ wiederhergestellt (Cmd/Strg+Umschalt+Enter, Doppelklick auf die Kopfzeile)
    pub fn umschalten_maximiert(&mut self, b: Bereich) { if self.max == Some(b) { self.max = None } else { self.max = Some(b) } }
    /// Auf die Vorgabe zurück („Layout zurücksetzen“)
    pub fn zuruecksetzen(&mut self) { *self = Layout { vor_debug: self.vor_debug.clone(), ..Layout::vorgabe() } }

    /// Zen-Modus (Cmd/Strg+Umschalt+B): Seitenleiste, Schublade und Panel klappen gemeinsam ein, eine Maximierung endet; erneut schaltet es das vorige Layout wieder her
    pub fn zen_umschalten(&mut self) {
        match self.zen.take() {
            Some((min, max)) => { self.min = min; self.max = max }
            None => { self.zen = Some((self.min, self.max)); self.max = None; for b in [Bereich::Seitenleiste, Bereich::Schublade, Bereich::Panel] { self.min[b.index()] = true } }
        }
    }
    pub fn zen(&self) -> bool { self.zen.is_some() }

    /// Debug-Sitzung beginnt: eine Maximierung endet, Schublade und Panel klappen auf; das vorige Layout wird gemerkt
    pub fn debug_start(&mut self) {
        if let Some((min, max)) = self.zen.take() { self.min = min; self.max = max } // Zen endet: eine Debug-Sitzung braucht Schublade und Panel
        if self.vor_debug.is_none() { self.vor_debug = Some((self.min, self.max)) }
        self.max = None; self.min[Bereich::Schublade.index()] = false;
    }
    // Entscheidung 23: die Debug-Fläche ist ein Reiter der Schublade; das rechte Panel bleibt, wie es ist.
    /// Debug-Sitzung endet: das Layout davor kehrt zurück (Änderungen während der Sitzung an der Maximierung bleiben nicht bestehen)
    pub fn debug_ende(&mut self) { if let Some((min, max)) = self.vor_debug.take() { self.min = min; self.max = max } }
    pub fn in_debug(&self) -> bool { self.vor_debug.is_some() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Bereich::*;

    #[test] fn vorgabe_panel_ist_ein_streifen() {
        let l = Layout::vorgabe();
        assert_eq!((l.anzeige(Seitenleiste), l.anzeige(Dokument), l.anzeige(Schublade), l.anzeige(Panel)), (Anzeige::Normal, Anzeige::Normal, Anzeige::Normal, Anzeige::Minimiert));
        assert!(!l.offen(Panel) && l.offen(Dokument) && l.maximiert().is_none());
    }
    #[test] fn minimieren_und_aufklappen_je_bereich() {
        let mut l = Layout::vorgabe();
        assert!(l.minimieren(Seitenleiste) && l.anzeige(Seitenleiste) == Anzeige::Minimiert);
        l.umschalten(Seitenleiste); assert_eq!(l.anzeige(Seitenleiste), Anzeige::Normal);
        l.umschalten(Panel); assert!(l.offen(Panel)); l.umschalten(Panel); assert!(!l.offen(Panel));
    }
    #[test] fn das_dokumentenfeld_bleibt_immer_offen() {
        let mut l = Layout::vorgabe();
        assert!(!l.kann_minimieren(Dokument) && !l.minimieren(Dokument)); assert_eq!(l.anzeige(Dokument), Anzeige::Normal);
        for b in [Seitenleiste, Schublade, Panel] { l.minimieren(b); }
        assert!(l.offen(Dokument), "nie alles zu");
    }
    #[test] fn maximieren_klappt_die_uebrigen_ein_und_wiederherstellen_stellt_her() {
        let mut l = Layout::vorgabe(); l.minimieren(Seitenleiste);
        l.maximieren(Schublade);
        assert_eq!((l.anzeige(Schublade), l.anzeige(Dokument), l.anzeige(Seitenleiste), l.anzeige(Panel)), (Anzeige::Maximiert, Anzeige::Minimiert, Anzeige::Minimiert, Anzeige::Minimiert));
        l.wiederherstellen();
        assert_eq!((l.anzeige(Schublade), l.anzeige(Seitenleiste), l.anzeige(Panel)), (Anzeige::Normal, Anzeige::Minimiert, Anzeige::Minimiert), "vorheriger Stand, auch die schon minimierte Spalte");
    }
    #[test] fn umschalten_maximiert_und_wechsel_des_maximierten() {
        let mut l = Layout::vorgabe();
        l.umschalten_maximiert(Dokument); assert_eq!(l.maximiert(), Some(Dokument));
        l.umschalten_maximiert(Schublade); assert_eq!(l.maximiert(), Some(Schublade), "ein anderer Bereich löst den vorigen ab");
        l.umschalten_maximiert(Schublade); assert_eq!(l.maximiert(), None);
    }
    #[test] fn ein_eingeklappter_bereich_oeffnet_sich_aus_dem_maximierten_zustand() {
        let mut l = Layout::vorgabe(); l.maximieren(Dokument);
        l.aufklappen(Seitenleiste); assert!(l.maximiert().is_none() && l.offen(Seitenleiste) && l.offen(Schublade), "Aufklappen beendet die Maximierung");
        let mut l = Layout::vorgabe(); l.maximieren(Schublade); l.minimieren(Schublade);
        assert!(l.maximiert().is_none() && l.anzeige(Schublade) == Anzeige::Minimiert, "Minimieren beendet das eigene Maximieren");
    }
    #[test] fn debug_oeffnet_die_schublade_laesst_das_panel_und_endet_mit_dem_alten_layout() {
        let mut l = Layout::vorgabe(); l.minimieren(Schublade); l.maximieren(Dokument);
        l.debug_start();
        assert!(l.in_debug() && l.maximiert().is_none() && l.offen(Schublade) && !l.offen(Panel), "Schublade offen, Panel unberührt (minimiert wie zuvor)");
        l.maximieren(Schublade); l.debug_start(); assert!(l.maximiert().is_none(), "ein zweiter Start überschreibt das gemerkte Layout nicht");
        l.debug_ende();
        assert!(!l.in_debug());
        assert_eq!((l.maximiert(), l.anzeige(Panel)), (Some(Dokument), Anzeige::Minimiert), "Layout wie vor der Sitzung");
    }
    #[test] fn gemerkt_enthaelt_nie_den_maximierten_zustand() {
        let mut l = Layout::vorgabe(); l.minimieren(Seitenleiste); l.aufklappen(Panel); l.maximieren(Schublade);
        let g = l.gemerkt(); assert_eq!(g, Gemerkt { seitenleiste_minimiert: true, schublade_minimiert: false, panel_minimiert: false });
        let n = Layout::aus_gemerkt(g); assert!(n.maximiert().is_none() && n.anzeige(Seitenleiste) == Anzeige::Minimiert && n.offen(Panel));
        let j = serde_json::to_string(&g).unwrap(); assert_eq!(serde_json::from_str::<Gemerkt>(&j).unwrap(), g);
        assert_eq!(serde_json::from_str::<Gemerkt>("{}").unwrap(), Gemerkt::default(), "fehlende Felder: Vorgabe");
    }
    #[test] fn zuruecksetzen_stellt_die_vorgabe_her() {
        let mut l = Layout::vorgabe(); l.minimieren(Seitenleiste); l.aufklappen(Panel); l.maximieren(Dokument);
        l.zuruecksetzen(); assert_eq!(l, Layout::vorgabe());
    }
    #[test] fn zen_klappt_alles_ein_stellt_her_und_wird_nie_gemerkt() {
        let mut l = Layout::vorgabe(); l.minimieren(Seitenleiste); l.maximieren(Schublade);
        let vorher = l.gemerkt(); l.zen_umschalten();
        assert!(l.zen() && l.maximiert().is_none() && !l.offen(Seitenleiste) && !l.offen(Schublade) && !l.offen(Panel) && l.offen(Dokument), "nur das Dokumentenfeld bleibt");
        assert_eq!(l.gemerkt(), vorher, "gemerkt wird das Layout davor");
        l.zen_umschalten(); assert!(!l.zen() && !l.offen(Seitenleiste) && l.maximiert() == Some(Schublade), "vorheriger Stand: Seitenleiste zu, Schublade maximiert");
        l.zen_umschalten(); l.debug_start(); assert!(!l.zen() && l.offen(Schublade), "Debug beendet Zen");
    }
}
