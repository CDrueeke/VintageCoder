// SPDX-License-Identifier: GPL-3.0-or-later
//! Stabile Bild-Handles für die Studios. `Handle::from_rgba` vergibt bei **jedem** Aufruf eine neue Kennung, und der Renderer lädt das Bild dann
//! neu als Textur hoch (und wirft die alte weg): wird das Bild bei jedem Aufbau der Ansicht (auch bei jeder Mausbewegung) neu verpackt, flackert die
//! Fläche. Hier wird das Handle nach dem Inhalt (Größe und Pixel) wiederverwendet; ein neues entsteht nur, wenn sich das Bild wirklich ändert.
use iced::widget::image::Handle;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const MAX: usize = 64;
thread_local! {
    static HANDLES: RefCell<HashMap<u64, (u64, Handle)>> = RefCell::new(HashMap::new());
    static ZEIT: Cell<u64> = const { Cell::new(0) };
    static ERZEUGT: Cell<usize> = const { Cell::new(0) };
}

/// Handle zu den Pixeln (RGBA); gleicher Inhalt liefert dasselbe Handle
pub fn handle(w: u32, h: u32, rgba: &[u8]) -> Handle {
    let mut hs = std::collections::hash_map::DefaultHasher::new(); (w, h).hash(&mut hs); rgba.hash(&mut hs); let key = hs.finish();
    let jetzt = ZEIT.with(|z| { z.set(z.get() + 1); z.get() });
    HANDLES.with(|c| {
        let mut c = c.borrow_mut();
        if let Some(e) = c.get_mut(&key) { e.0 = jetzt; return e.1.clone() }
        if c.len() >= MAX { if let Some(alt) = c.iter().min_by_key(|(_, v)| v.0).map(|(k, _)| *k) { c.remove(&alt); } }
        let neu = Handle::from_rgba(w, h, rgba.to_vec()); ERZEUGT.with(|z| z.set(z.get() + 1)); c.insert(key, (jetzt, neu.clone())); neu
    })
}
/// Wie viele Handles seit dem Start neu erzeugt wurden (für den Regressionstest: Mausbewegungen dürfen keine erzeugen)
pub fn erzeugt() -> usize { ERZEUGT.with(|z| z.get()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn gleicher_inhalt_gleiches_handle_anderer_inhalt_neues() {
        let a = vec![1u8, 2, 3, 255]; let e0 = erzeugt();
        let h1 = handle(1, 1, &a); let h2 = handle(1, 1, &a); assert_eq!(h1.id(), h2.id(), "gleicher Inhalt: dasselbe Handle"); assert_eq!(erzeugt(), e0 + 1, "nur ein Handle erzeugt");
        let h3 = handle(1, 1, &[9, 9, 9, 255]); assert_ne!(h1.id(), h3.id()); assert_eq!(erzeugt(), e0 + 2);
    }
}
