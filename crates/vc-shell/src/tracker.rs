// SPDX-License-Identifier: GPL-3.0-or-later
//! Tracker (E5): `SongDoc` (vc-ide-model) als Dokument im `DocStore`. Die Ansicht liefert das Raster des gewählten Musters (Zellen als Text „NOT II“), Cursor und Auswahl, Reihenfolge,
//! Instrumente mit allen Feldern, Tempo, Oktave und Schrittweite; Absichten (`apply`) sind die Operationen des Modells (Eingabe, Block, Muster, Reihenfolge, Instrumente). Die Wiedergabe folgt in E5.2.
use crate::docstore::{Dokument, ExportQuelle, Fehler};
use serde_json::{json, Value};
use std::path::Path;
use vc_ide_model::songdoc::{Eingabe, SongDoc};
use vc_ide_model::synth::{parse_cell, NOTE_OFF};

/// `spielt`: dieses Dokument hat die Wiedergabe gestartet (`Some(true)` = nur das gewählte Muster, einmal)
pub struct TrackerDok { pub doc: SongDoc, spielt: Option<bool> }

impl Drop for TrackerDok { fn drop(&mut self) { if self.spielt.is_some() { crate::audio::stoppen() } } }

impl TrackerDok {
    fn spielt_noch(&self) -> bool { self.spielt.is_some() && crate::audio::spielt() }
    /// Noten- und Instrumentenänderungen während der Wiedergabe sofort hörbar machen
    fn nachziehen(&self) { if let (true, Some(m)) = (self.spielt_noch(), self.spielt) { crate::audio::aktualisieren(self.doc.song_daten(m)) } }
    fn vorhoeren(&self, note: u8, i: usize) { if let Some(ins) = self.doc.instrumente().get(i) { crate::audio::vorhoeren(ins, note) } }
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<TrackerDok, Fehler> {
        let doc = SongDoc::oeffnen(root, rel, data).map_err(|e| Fehler::mit("song-unlesbar", json!({ "detail": e, "rel": rel })))?;
        Ok(TrackerDok { doc, spielt: None })
    }
    pub fn doc_view(&self) -> Value { Dokument::view(self) }
}

impl Dokument for TrackerDok {
    fn art(&self) -> &'static str { "song" }
    fn view(&self) -> Value {
        let d = &self.doc; let (zn, kn) = (d.zeilen(), d.kanaele);
        let grid: Vec<Vec<String>> = (0..zn).map(|r| (0..kn).map(|c| d.zelle(d.muster, r, c)).collect()).collect();
        let ins: Vec<Value> = (0..d.anzahl_instrumente()).map(|i| { let mut v = d.media["instruments"][i].clone(); if let Some(o) = v.as_object_mut() { o.insert("used".into(), json!(d.instrument_benutzt(i))); } v }).collect();
        let sel = d.auswahl.map(|((r0, c0), (r1, c1))| json!([r0, c0, r1, c1]));
        let ord = d.reihenfolge();
        json!({
            "rel": d.rel, "kind": if d.ist_play() { "play" } else { "song" }, "title": Path::new(&d.rel).file_name().map_or_else(|| d.rel.clone(), |x| x.to_string_lossy().into_owned()), "name": d.media["name"].as_str().unwrap_or(""),
            "target": d.target, "chip": d.chip, "pokey": d.pokey, "channels": kn, "rows": zn, "hz": d.hz, "pattern": d.muster, "patternCount": d.anzahl_muster(), "grid": grid,
            "cursor": [d.cursor.0, d.cursor.1], "sel": sel, "order": ord, "ordSel": d.ord_wahl, "loop": d.media["loop"].as_u64().unwrap_or(0), "tempo": d.tempo(), "volume": d.lautstaerke(),
            "octave": d.oktave, "instrument": d.instrument, "step": d.schritt, "instruments": ins, "insSel": d.ins_wahl, "maxInstruments": d.max_instrumente(),
            "mute": d.stumm, "solo": d.solo, "spielt": self.spielt_noch(), "spielModus": self.spielt.map(|m| if m { "pattern" } else { "song" }), "hasClip": d.zwischen.is_some(), "canUndo": d.kann_undo(), "canRedo": d.kann_redo(), "dirty": d.dirty(), "status": d.status,
        })
    }
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
        self.doc.status.clear();
        let n = |k: &str| op[k].as_u64().map(|x| x as usize).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let z = |k: &str| op[k].as_i64().map(|x| x as i32).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let (zn, kn) = (self.doc.zeilen(), self.doc.kanaele);
        let mut vorhoeren: Value = Value::Null;
        match op["kind"].as_str().unwrap_or("") {
            "cursor" => { let (r, c) = (n("r")?, n("c")?); if r >= zn || c >= kn { return Err(Fehler::neu("op-ungueltig")) } self.doc.setze_cursor(r, c) }
            "move" => self.doc.bewege(z("dr")?, z("dc")?, op["extend"].as_bool().unwrap_or(false)),
            "jump" => self.doc.springe(n("r")?),
            "select" => { let (a, b) = ((n("r0")?, n("c0")?), (n("r1")?, n("c1")?)); if a.0 >= zn || b.0 >= zn || a.1 >= kn || b.1 >= kn { return Err(Fehler::neu("op-ungueltig")) } self.doc.waehle(a, b) }
            "select_all" => self.doc.alles_waehlen(),
            "pattern" => { let p = n("p")?; if p >= self.doc.anzahl_muster() { return Err(Fehler::neu("op-ungueltig")) } self.doc.muster_waehlen(p) }
            "octave" => self.doc.oktave_aendern(z("d")?),
            "instrument_cur" => self.doc.instrument_aendern(z("d")?),
            "step" => self.doc.schritt = n("n")?.clamp(0, 16),
            "note" => { let h = n("halbton")?; if h > 24 { return Err(Fehler::neu("op-ungueltig")) } if let Eingabe::Note(note, i) = self.doc.note_eingeben(h as u8) { self.vorhoeren(note, i as usize); vorhoeren = json!({ "note": note, "instrument": i }) } }
            "note_end" => self.doc.notenende(),
            "clear" => self.doc.loeschen(),
            "apply_instrument" => self.doc.instrument_anwenden(),
            "cell_instrument" => { if let Some((note, i)) = self.doc.zell_instrument_aendern(z("dx")?) { self.vorhoeren(note, i as usize); vorhoeren = json!({ "note": note, "instrument": i }) } }
            "copy" => self.doc.kopieren(),
            "cut" => self.doc.ausschneiden(),
            "paste" => { if self.doc.zwischen.is_none() { return Err(Fehler::neu("song-zwischenablage")) } self.doc.einfuegen() }
            "transpose" => self.doc.transponieren(z("n")?),
            "pattern_new" => self.doc.muster_neu(),
            "pattern_dup" => self.doc.muster_duplizieren(),
            "pattern_delete" => self.doc.muster_loeschen(),
            "ord_select" => self.doc.ord_waehlen(n("i")?),
            "ord_append" => self.doc.ord_anhaengen(),
            "ord_remove" => self.doc.ord_entfernen(),
            "ord_set" => self.doc.ord_setzen(),
            "ord_move" => self.doc.ord_verschieben(z("d")?),
            "loop" => self.doc.schleife_setzen(n("l")?),
            "ins_new" => self.doc.instrument_neu(),
            "ins_dup" => self.doc.instrument_duplizieren(),
            "ins_delete" => { if !self.doc.instrument_loeschen() { return Err(Fehler::neu("song-instrument-loeschen")) } }
            "ins_select" => { let i = n("i")?; if i >= self.doc.anzahl_instrumente() { return Err(Fehler::neu("op-ungueltig")) } self.doc.instrument_waehlen(i) }
            "ins_value" => { let k = op["key"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "key" })))?; if !["name", "wave", "pw", "a", "d", "s", "r", "dist", "loop"].contains(&k) { return Err(Fehler::neu("op-ungueltig")) } self.doc.instrument_wert(k, op["value"].clone(), op["first"].as_bool().unwrap_or(true)) }
            "envelope" => { let (k, v) = (n("k")?, n("v")?); if k > 15 || v > 15 { return Err(Fehler::neu("op-ungueltig")) } self.doc.huellkurve_setzen(k, v as u8, op["first"].as_bool().unwrap_or(true)) }
            "rows" => { let r = n("n")?; if !(1..=128).contains(&r) { return Err(Fehler::neu("op-ungueltig")) } self.doc.zeilen_setzen(r) }
            "tempo" => self.doc.tempo_setzen(n("t")? as u64),
            "volume" => self.doc.lautstaerke_setzen(n("v")? as u64),
            "mute" => { let c = n("c")?; self.doc.stumm_umschalten(c); crate::audio::stumm_solo(c, self.doc.stumm[c], self.doc.solo[c]) }
            "solo" => { let c = n("c")?; self.doc.solo_umschalten(c); crate::audio::stumm_solo(c, self.doc.stumm[c], self.doc.solo[c]) }
            "vorschau" => { let h = n("halbton")?; if h > 24 { return Err(Fehler::neu("op-ungueltig")) } let note = self.doc.oktave as usize * 12 + h + 1; if note < 255 { self.vorhoeren(note as u8, self.doc.instrument); vorhoeren = json!({ "note": note, "instrument": self.doc.instrument }) } }
            "play" => {
                let nur_muster = match op["mode"].as_str().unwrap_or("song") { "pattern" => true, "song" => false, _ => return Err(Fehler::neu("op-ungueltig")) };
                let (ord, zeile) = if nur_muster || self.doc.ist_play() { (0, if nur_muster { self.doc.cursor.0 } else { 0 }) } else { (self.doc.ord_wahl, 0) };
                crate::audio::spielen(self.doc.song_daten(nur_muster), ord, zeile, &self.doc.stumm, &self.doc.solo).map_err(|e| Fehler::mit("audio-fehler", json!({ "detail": e })))?;
                self.spielt = Some(nur_muster);
            }
            "stop" => { if self.spielt.is_some() { crate::audio::stoppen() } self.spielt = None }
            "refresh" => {}
            "undo" => { if !self.doc.kann_undo() { return Err(Fehler::neu("undo-leer")) } self.doc.rueckgaengig() }
            "redo" => { if !self.doc.kann_redo() { return Err(Fehler::neu("redo-leer")) } self.doc.wiederholen() }
            "save" => self.save()?,
            _ => return Err(Fehler::neu("op-ungueltig")),
        }
        if !matches!(op["kind"].as_str().unwrap_or(""), "play" | "stop" | "refresh" | "vorschau" | "cursor" | "move" | "jump" | "select" | "select_all" | "mute" | "solo") { self.nachziehen() }
        if self.spielt.is_some() && !crate::audio::spielt() { self.spielt = None }
        let mut v = self.view(); v["audition"] = vorhoeren; let _ = (parse_cell, NOTE_OFF); Ok(v)
    }
    fn undo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "undo" })) }
    fn redo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "redo" })) }
    fn save(&mut self) -> Result<(), Fehler> { self.doc.speichern().map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e }))) }
    fn undo_tiefe(&self) -> usize { if self.doc.kann_undo() { 1 } else { 0 } }
    fn export_quelle(&self) -> Option<ExportQuelle> {
        let rel = &self.doc.rel; let name = Path::new(rel).file_stem().map_or_else(|| "song".into(), |s| s.to_string_lossy().into_owned());
        Some(ExportQuelle { rel: rel.clone(), kind: if self.doc.ist_play() { "play".into() } else { "song".into() }, name, media_text: self.doc.media_text() })
    }
    fn ungesichert(&self) -> bool { self.doc.dirty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn kopie(name: &str, bsp: &str) -> (std::path::PathBuf, TrackerDok) {
        let w = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let d = std::env::temp_dir().join(format!("vc_trk_{name}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("music")).unwrap();
        let f = std::path::Path::new(bsp).file_name().unwrap().to_string_lossy().into_owned(); std::fs::copy(w.join(bsp), d.join("music").join(&f)).unwrap();
        let k = TrackerDok::oeffnen(&d, &format!("music/{f}"), &w.join("data")).unwrap(); (d, k)
    }
    #[test] fn ansicht_hat_raster_reihenfolge_und_instrumente() {
        let (d, k) = kopie("ansicht", "examples/mega65/jumprun/music/tune.vcsong"); let v = k.view();
        assert_eq!(v["kind"], "song"); assert!(v["channels"].as_u64().unwrap() >= 3); assert_eq!(v["grid"].as_array().unwrap().len() as u64, v["rows"].as_u64().unwrap()); assert!(!v["order"].as_array().unwrap().is_empty()); assert!(!v["instruments"].as_array().unwrap().is_empty()); assert_eq!(v["dirty"], false);
        std::fs::remove_dir_all(d).ok();
    }
    #[test] fn eingabe_block_muster_reihenfolge_und_instrumente() {
        let (d, mut k) = kopie("ops", "examples/mega65/jumprun/music/tune.vcsong");
        k.apply(&json!({ "kind": "cursor", "r": 2, "c": 1 })).unwrap(); let v = k.apply(&json!({ "kind": "note", "halbton": 4 })).unwrap(); assert_eq!(v["audition"]["instrument"], 0); assert_ne!(v["grid"][2][1], "--- --"); assert_eq!(v["cursor"], json!([3, 1]), "Zeile weiter");
        let v = k.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["grid"][2][1], "--- --", "ein Rückgängig je Note"); k.apply(&json!({ "kind": "redo" })).unwrap();
        k.apply(&json!({ "kind": "select", "r0": 0, "c0": 0, "r1": 3, "c1": 1 })).unwrap(); k.apply(&json!({ "kind": "copy" })).unwrap(); k.apply(&json!({ "kind": "cursor", "r": 8, "c": 0 })).unwrap(); let v = k.apply(&json!({ "kind": "paste" })).unwrap(); assert!(v["hasClip"] == true);
        k.apply(&json!({ "kind": "transpose", "n": 12 })).unwrap(); k.apply(&json!({ "kind": "note_end" })).unwrap(); k.apply(&json!({ "kind": "clear" })).unwrap();
        let p0 = k.view()["patternCount"].as_u64().unwrap(); let v = k.apply(&json!({ "kind": "pattern_new" })).unwrap(); assert_eq!(v["patternCount"].as_u64().unwrap(), p0 + 1); k.apply(&json!({ "kind": "ord_append" })).unwrap(); let v = k.apply(&json!({ "kind": "pattern_dup" })).unwrap(); assert_eq!(v["patternCount"].as_u64().unwrap(), p0 + 2);
        let v = k.apply(&json!({ "kind": "pattern_delete" })).unwrap(); assert_eq!(v["patternCount"].as_u64().unwrap(), p0 + 1); k.apply(&json!({ "kind": "ord_move", "d": -1 })).unwrap(); k.apply(&json!({ "kind": "loop", "l": 0 })).unwrap();
        let ni = k.view()["instruments"].as_array().unwrap().len(); let v = k.apply(&json!({ "kind": "ins_new" })).unwrap(); assert_eq!(v["instruments"].as_array().unwrap().len(), ni + 1); k.apply(&json!({ "kind": "ins_value", "key": "name", "value": "bass" })).unwrap(); k.apply(&json!({ "kind": "ins_value", "key": "gibts", "value": 1 })).unwrap_err();
        let v = k.apply(&json!({ "kind": "ins_delete" })).unwrap(); assert_eq!(v["instruments"].as_array().unwrap().len(), ni, "letztes unbenutztes Instrument ist löschbar");
        let v = k.apply(&json!({ "kind": "rows", "n": 16 })).unwrap(); assert_eq!(v["rows"], 16); assert_eq!(k.apply(&json!({ "kind": "rows", "n": 0 })).unwrap_err().code, "op-ungueltig"); let v = k.apply(&json!({ "kind": "tempo", "t": 5 })).unwrap(); assert_eq!(v["tempo"], 5);
        assert_eq!(k.apply(&json!({ "kind": "cursor", "r": 999, "c": 0 })).unwrap_err().code, "op-ungueltig"); assert_eq!(k.apply(&json!({ "kind": "note", "halbton": 99 })).unwrap_err().code, "op-ungueltig");
        let v = k.apply(&json!({ "kind": "save" })).unwrap(); assert_eq!(v["dirty"], false); assert!(k.export_quelle().is_some());
        std::fs::remove_dir_all(d).ok();
    }
}
