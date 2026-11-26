// SPDX-License-Identifier: GPL-3.0-or-later
//! Messhilfen für Prüfpunkt G1: Echo-Befehl und ein kleines Pixelraster in Rust (24×21 wie ein Sprite) mit Delta-Antwort.
//! Zeigt, ob der Rundlauf „Oberfläche → Rust → Oberfläche“ für das Zeichnen reicht (Muster: lokaler Vorschaustrich, gebündelter Vorgang).
use serde_json::{json, Value};

pub const B: usize = 24;
pub const H: usize = 21;

pub struct Raster { pub zellen: Vec<u8>, pub tiefe: usize }
impl Default for Raster { fn default() -> Raster { Raster { zellen: vec![0; B * H], tiefe: 0 } } }

impl Raster {
    /// Ein Strich = eine Liste von Zellen `[x, y, farbe]`; Antwort: nur die tatsächlich geänderten Zellen (Delta) und die Tiefe des Verlaufs
    pub fn strich(&mut self, zellen: &[Value]) -> Value {
        let mut delta = vec![];
        for z in zellen {
            let (Some(x), Some(y), Some(c)) = (z[0].as_u64(), z[1].as_u64(), z[2].as_u64()) else { continue };
            let (x, y) = (x as usize, y as usize);
            if x < B && y < H && self.zellen[y * B + x] != c as u8 { self.zellen[y * B + x] = c as u8; delta.push(json!([x, y, c])); }
        }
        if !delta.is_empty() { self.tiefe += 1; }
        json!({ "cells": delta, "undoDepth": self.tiefe })
    }
    pub fn leeren(&mut self) { self.zellen.iter_mut().for_each(|z| *z = 0); self.tiefe = 0; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn strich_liefert_nur_geaenderte_zellen() {
        let mut r = Raster::default();
        let a = r.strich(&[json!([1, 2, 3]), json!([1, 2, 3]), json!([99, 0, 1])]);
        assert_eq!(a["cells"], json!([[1, 2, 3]])); assert_eq!(a["undoDepth"], 1);
        assert_eq!(r.strich(&[json!([1, 2, 3])])["cells"], json!([])); assert_eq!(r.tiefe, 1);
    }
}
