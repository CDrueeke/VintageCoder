// SPDX-License-Identifier: GPL-3.0-or-later
//! Reitergruppen des Editorbereichs (links, optional rechts): reine Datenlogik zum Verschieben, Teilen und Vereinen.

#[derive(Clone, Debug, PartialEq)]
pub struct Paar<T> { pub tabs: Vec<T>, pub aktiv: usize }
impl<T> Paar<T> { pub fn neu() -> Self { Paar { tabs: vec![], aktiv: 0 } } }

fn entfernt<T>(p: &mut Paar<T>, i: usize) -> T { let t = p.tabs.remove(i); if p.aktiv > i || p.aktiv >= p.tabs.len() { p.aktiv = p.aktiv.saturating_sub(1).min(p.tabs.len().saturating_sub(1)) } t }

/// Gibt es nach dem Entfernen eine leere Gruppe, rückt die andere nach links. Liefert, ob die rechte Gruppe noch besteht.
fn aufraeumen<T>(links: &mut Paar<T>, rechts: &mut Option<Paar<T>>) {
    if rechts.as_ref().is_some_and(|r| r.tabs.is_empty()) { *rechts = None }
    if links.tabs.is_empty() { if let Some(r) = rechts.take() { *links = r } }
}

/// Reiter `tab` in die Gruppe `ziel_rechts` setzen, vor den Reiter mit Index `vor` (Index vor dem Entfernen) oder ans Ende.
/// Die rechte Gruppe entsteht bei Bedarf; leere Gruppen verschwinden. Liefert, ob danach die rechte Gruppe den Fokus hat.
pub fn verschiebe<T: PartialEq>(links: &mut Paar<T>, rechts: &mut Option<Paar<T>>, tab: &T, ziel_rechts: bool, vor: Option<usize>) -> bool {
    let (von_rechts, i) = if let Some(i) = links.tabs.iter().position(|t| t == tab) { (false, i) } else if let Some(i) = rechts.as_ref().and_then(|r| r.tabs.iter().position(|t| t == tab)) { (true, i) } else { return rechts.is_some() && ziel_rechts };
    // Ziel gleich Quelle und einziger Reiter ohne Teilung: nichts zu tun
    if !ziel_rechts && !von_rechts && vor.is_none() && links.tabs.len() == 1 { return false }
    if ziel_rechts && rechts.is_none() && links.tabs.len() == 1 { return false } // der letzte Reiter bliebe links leer
    let t = if von_rechts { entfernt(rechts.as_mut().unwrap(), i) } else { entfernt(links, i) };
    if ziel_rechts && rechts.is_none() { *rechts = Some(Paar::neu()) }
    let ziel_gleich = ziel_rechts == von_rechts;
    let ziel = if ziel_rechts { rechts.as_mut().unwrap() } else { &mut *links };
    let idx = vor.map(|v| if ziel_gleich && i < v { v - 1 } else { v }).unwrap_or(ziel.tabs.len()).min(ziel.tabs.len());
    ziel.tabs.insert(idx, t); ziel.aktiv = idx;
    aufraeumen(links, rechts);
    rechts.is_some() && ziel_rechts
}

/// Aktiven Reiter der linken Gruppe in eine rechte verschieben (Teilen) oder beide Gruppen vereinen. Liefert, ob danach rechts fokussiert ist.
pub fn teile_oder_vereine<T: PartialEq + Clone>(links: &mut Paar<T>, rechts: &mut Option<Paar<T>>, fokus_rechts: bool) -> bool {
    if let Some(r) = rechts.take() {
        let aktiv_tab = if fokus_rechts { r.tabs.get(r.aktiv).cloned() } else { links.tabs.get(links.aktiv).cloned() };
        links.tabs.extend(r.tabs);
        if let Some(a) = aktiv_tab { links.aktiv = links.tabs.iter().position(|t| *t == a).unwrap_or(0) }
        return false;
    }
    let Some(t) = links.tabs.get(links.aktiv).cloned() else { return false };
    verschiebe(links, rechts, &t, true, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(v: &[&'static str], a: usize) -> Paar<&'static str> { Paar { tabs: v.to_vec(), aktiv: a } }
    #[test] fn ziehen_an_den_rechten_rand_teilt() {
        let (mut l, mut r) = (p(&["a", "b", "c"], 0), None);
        assert!(verschiebe(&mut l, &mut r, &"b", true, None));
        assert_eq!(l.tabs, ["a", "c"]); assert_eq!(r, Some(p(&["b"], 0)));
        assert_eq!(l.aktiv, 0, "aktiver Reiter bleibt gültig");
    }
    #[test] fn einziger_reiter_laesst_sich_nicht_abspalten() {
        let (mut l, mut r) = (p(&["a"], 0), None);
        assert!(!verschiebe(&mut l, &mut r, &"a", true, None)); assert_eq!(l.tabs, ["a"]); assert!(r.is_none());
    }
    #[test] fn umordnen_in_derselben_gruppe() {
        let (mut l, mut r) = (p(&["a", "b", "c"], 2), None);
        verschiebe(&mut l, &mut r, &"a", false, Some(2)); assert_eq!(l.tabs, ["b", "a", "c"], "a vor c (Index 2 vor dem Entfernen)"); assert_eq!(l.aktiv, 1);
        verschiebe(&mut l, &mut r, &"c", false, Some(0)); assert_eq!(l.tabs, ["c", "b", "a"]);
        verschiebe(&mut l, &mut r, &"c", false, None); assert_eq!(l.tabs, ["b", "a", "c"], "ohne Ziel ans Ende");
    }
    #[test] fn zwischen_gruppen_und_leere_gruppe_verschwindet() {
        let (mut l, mut r) = (p(&["a", "b"], 0), None);
        verschiebe(&mut l, &mut r, &"b", true, None);
        assert!(!verschiebe(&mut l, &mut r, &"b", false, Some(0)), "rechte Gruppe leer: weg, Fokus links");
        assert_eq!(l.tabs, ["b", "a"]); assert!(r.is_none());
        let (mut l, mut r) = (p(&["a", "b"], 1), None); verschiebe(&mut l, &mut r, &"a", true, None);
        // jetzt links: [b], rechts: [a]; den letzten linken Reiter nach rechts ziehen: links leer → rechts rückt nach links
        let f = verschiebe(&mut l, &mut r, &"b", true, Some(0)); assert!(!f); assert_eq!(l.tabs, ["b", "a"]); assert!(r.is_none());
    }
    #[test] fn teilen_und_vereinen() {
        let (mut l, mut r) = (p(&["a", "b", "c"], 1), None);
        assert!(teile_oder_vereine(&mut l, &mut r, false)); assert_eq!(l.tabs, ["a", "c"]); assert_eq!(r, Some(p(&["b"], 0)));
        assert!(!teile_oder_vereine(&mut l, &mut r, true)); assert!(r.is_none()); assert_eq!(l.tabs, ["a", "c", "b"]); assert_eq!(l.aktiv, 2, "aktiver Reiter der fokussierten Gruppe bleibt aktiv");
    }
}
