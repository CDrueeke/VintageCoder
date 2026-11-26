// SPDX-License-Identifier: GPL-3.0-or-later
//! Tracker-Song (`.vcsong`, Art `song`) ohne Oberfläche: Muster-Raster mit Zellen „NOT II“ (Note, Instrument), Cursor und Navigation,
//! Noteneingabe per Klaviertasten, Auswahl und Block (kopieren, ausschneiden, einfügen, transponieren), Oktave und Instrument,
//! Reihenfolge der Muster, Zeilenzahl, Tempo, Rückgängig, Speichern. Die Wiedergabe (`synth::Engine`) bekommt ihre Daten aus [`SongDoc::song_daten`].
use crate::synth::{fmt_cell, parse_cell, Instr, SongData, INS_KEEP, NOTE_OFF};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Klaviertaste → Halbtöne über dem C der Oktave (zwei Reihen wie am Tracker: Z S X D C V G B H N J M, dann Q 2 W 3 E R 5 T 6 Y 7 U I)
pub fn klaviertaste(c: char) -> Option<u8> {
    Some(match c.to_ascii_lowercase() { 'z' => 0, 's' => 1, 'x' => 2, 'd' => 3, 'c' => 4, 'v' => 5, 'g' => 6, 'b' => 7, 'h' => 8, 'n' => 9, 'j' => 10, 'm' => 11,
        'q' => 12, '2' => 13, 'w' => 14, '3' => 15, 'e' => 16, 'r' => 17, '5' => 18, 't' => 19, '6' => 20, 'y' => 21, '7' => 22, 'u' => 23, 'i' => 24, _ => return None })
}

/// Rechteckige Auswahl im Raster: (Zeile, Kanal) von … bis, beide eingeschlossen
pub type Auswahl = ((usize, usize), (usize, usize));

pub struct SongDoc {
    pub rel: String, abs: PathBuf,
    pub media: Value, pub target: String, pub chip: String, pub kanaele: usize, pub pokey: bool,
    /// Bildrate des Zielsystems (Takt der Hüllkurven und der Zeilendauer): 50 oder 60
    pub hz: f32,
    undo: Vec<String>, redo: Vec<String>, saved: String, pub status: String,
    /// Cursor (Zeile, Kanal), gewähltes Muster, Oktave, Instrument, Schrittweite nach der Eingabe
    pub cursor: (usize, usize), pub muster: usize, pub oktave: u8, pub instrument: usize, pub schritt: usize,
    pub auswahl: Option<Auswahl>, anker: Option<(usize, usize)>,
    pub zwischen: Option<Vec<Vec<String>>>,
    /// gewählter Eintrag der Reihenfolge und gewähltes Instrument im Editor; Stumm/Solo je Kanal (für die Wiedergabe)
    pub ord_wahl: usize, pub ins_wahl: usize, pub stumm: Vec<bool>, pub solo: Vec<bool>,
}

pub enum Eingabe { Keine, /** Note und Instrument zum Vorhören */ Note(u8, usize) }

impl SongDoc {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<SongDoc, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| vc_i18n::t!("songdoc-rel-e", rel = &rel, e = e.to_string()))?).map_err(|e| vc_i18n::t!("songdoc-rel-e", rel = &rel, e = e.to_string()))?;
        if !matches!(media["kind"].as_str(), Some("song" | "play")) { return Err(vc_i18n::t!("songdoc-rel-kein-song-und-kein-play", rel = &rel)) }
        let profiles = vc_media::load_profiles(data)?; let p = vc_media::profile(&profiles, media["profile"].as_str().unwrap_or(""))?;
        let play = media["kind"] == "play"; let chip = p.raw["chip"].as_str().unwrap_or("sid").to_string(); let kanaele = if play { media["voices"].as_u64().unwrap_or(6) as usize } else { p.raw["channels"].as_u64().unwrap_or(3) as usize };
        let hz = if media["clock"].as_str() == Some("ntsc") { 60.0 } else { 50.0 };
        Ok(SongDoc { rel: rel.into(), abs, saved: media.to_string(), media, target: p.target.clone(), pokey: matches!(chip.as_str(), "pokey" | "ted" | "vic"), chip, kanaele, hz,
            undo: vec![], redo: vec![], status: String::new(), cursor: (0, 0), muster: 0, oktave: 3, instrument: 0, schritt: 1, auswahl: None, anker: None, zwischen: None, ord_wahl: 0, ins_wahl: 0, stumm: vec![false; kanaele], solo: vec![false; kanaele] })
    }

    // -- Lesen ----------------------------------------------------------------------------------------------------------
    /// PLAY-Tracker (BASIC-65-Befehl PLAY): ein Muster `steps`, `tempo`, `volume`, Hüllkurven wie SID
    pub fn ist_play(&self) -> bool { self.media["kind"] == "play" }
    pub fn zeilen(&self) -> usize { if self.ist_play() { self.media["steps"].as_array().map_or(1, |a| a.len()).max(1) } else { self.media["rows"].as_u64().unwrap_or(32).max(1) as usize } }
    pub fn anzahl_muster(&self) -> usize { if self.ist_play() { 1 } else { self.media["patterns"].as_array().map_or(0, |a| a.len()) } }
    pub fn zelle(&self, p: usize, r: usize, c: usize) -> String { if self.ist_play() { self.media["steps"][r][c].as_str().unwrap_or("--- --").to_string() } else { self.media["patterns"][p]["rows"][r][c].as_str().unwrap_or("--- --").to_string() } }
    pub fn reihenfolge(&self) -> Vec<usize> { if self.ist_play() { return vec![0] } self.media["order"].as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0) as usize).collect()).unwrap_or_default() }
    pub fn tempo(&self) -> u64 { if self.ist_play() { self.media["tempo"].as_u64().unwrap_or(8) } else { self.media["speed"].as_u64().unwrap_or(6) } }
    pub fn lautstaerke(&self) -> u64 { self.media["volume"].as_u64().unwrap_or(8) }
    pub fn lautstaerke_setzen(&mut self, v: u64) { self.undo_punkt(); self.media["volume"] = json!(v.min(15)) }
    pub fn anzahl_instrumente(&self) -> usize { self.media["instruments"].as_array().map_or(0, |a| a.len()) }
    pub fn instrument_name(&self, i: usize) -> String { self.media["instruments"][i]["name"].as_str().unwrap_or("").to_string() }
    pub fn instrumente(&self) -> Vec<Instr> { self.media["instruments"].as_array().map(|a| a.iter().map(|i| Instr::from_json(i, self.pokey)).collect()).unwrap_or_default() }
    /// Zielzeichen für die Kopfzeile je Kanal
    pub fn kanal_name(&self, c: usize) -> String { format!("{}", c + 1) }

    /// Daten für den Sequenzer; `nur_muster`: nur das gewählte Muster, ohne Wiederholung der Reihenfolge
    pub fn song_daten(&self, nur_muster: bool) -> SongData {
        if self.ist_play() {
            let rows: Vec<Vec<(u8, u8)>> = self.media["steps"].as_array().map(|ss| ss.iter().map(|r| r.as_array().map(|cs| cs.iter().map(|c| parse_cell(c.as_str().unwrap_or(""))).collect()).unwrap_or_default()).collect()).unwrap_or_default();
            return SongData { pokey: false, channels: self.kanaele, rows: rows.len().max(1), order: vec![0], loop_at: 0, patterns: vec![rows], instruments: self.instrumente(), row_secs: 1.0 / self.tempo().max(1) as f32, frame_hz: 50.0 };
        }
        let patterns = self.media["patterns"].as_array().map(|ps| ps.iter().map(|p| p["rows"].as_array().map(|rows| rows.iter().map(|r| r.as_array().map(|cs| cs.iter().map(|c| parse_cell(c.as_str().unwrap_or(""))).collect()).unwrap_or_default()).collect()).unwrap_or_default()).collect()).unwrap_or_default();
        SongData { pokey: self.pokey, channels: self.kanaele, rows: self.zeilen(), order: if nur_muster { vec![self.muster] } else { self.reihenfolge() }, loop_at: if nur_muster { usize::MAX } else { self.media["loop"].as_u64().unwrap_or(0) as usize },
            patterns, instruments: self.instrumente(), row_secs: self.tempo() as f32 / self.hz, frame_hz: self.hz }
    }

    // -- Zustand --------------------------------------------------------------------------------------------------------
    pub fn dirty(&self) -> bool { self.media.to_string() != self.saved }
    fn undo_punkt(&mut self) { self.undo.push(self.media.to_string()); if self.undo.len() > 200 { self.undo.remove(0); } self.redo.clear() }
    fn nach_aenderung(&mut self) { self.cursor.0 = self.cursor.0.min(self.zeilen() - 1); self.muster = self.muster.min(self.anzahl_muster().saturating_sub(1)); self.instrument = self.instrument.min(self.anzahl_instrumente().saturating_sub(1)) }
    pub fn rueckgaengig(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; self.nach_aenderung() } } }
    pub fn wiederholen(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; self.nach_aenderung() } } }
    pub fn kann_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn kann_redo(&self) -> bool { !self.redo.is_empty() }
    pub fn media_text(&self) -> String { vc_media::project::format_media(&self.media) }
    pub fn speichern(&mut self) -> Result<(), String> {
        vc_core::atomar::schreiben(&self.abs, self.media_text()).map_err(|e| format!("{}: {e}", self.rel))?; self.saved = self.media.to_string(); self.status = vc_i18n::t!("songdoc-gespeichert"); Ok(())
    }

    // -- Navigation -----------------------------------------------------------------------------------------------------
    /// Cursor setzen (z. B. per Klick); löscht die Auswahl
    pub fn setze_cursor(&mut self, r: usize, c: usize) { self.cursor = (r.min(self.zeilen() - 1), c.min(self.kanaele.saturating_sub(1))); self.auswahl = None; self.anker = None }
    /// Cursor bewegen (mit Umlauf); mit `erweitern` wächst die Auswahl vom Ankerpunkt
    pub fn bewege(&mut self, dr: i32, dc: i32, erweitern: bool) {
        let (zn, kn) = (self.zeilen() as i32, self.kanaele.max(1) as i32);
        if erweitern && self.anker.is_none() { self.anker = Some(self.cursor) }
        let r = (self.cursor.0 as i32 + dr).rem_euclid(zn) as usize; let c = (self.cursor.1 as i32 + dc).rem_euclid(kn) as usize; self.cursor = (r, c);
        if erweitern { if let Some(a) = self.anker { self.auswahl = Some(((a.0.min(r), a.1.min(c)), (a.0.max(r), a.1.max(c)))) } } else { self.auswahl = None; self.anker = None }
    }
    pub fn springe(&mut self, r: usize) { self.cursor.0 = r.min(self.zeilen() - 1); self.auswahl = None; self.anker = None }
    /// Auswahl mit der Maus aufziehen
    pub fn waehle(&mut self, von: (usize, usize), bis: (usize, usize)) {
        let (zn, kn) = (self.zeilen() - 1, self.kanaele.saturating_sub(1)); let (a, b) = ((von.0.min(zn), von.1.min(kn)), (bis.0.min(zn), bis.1.min(kn)));
        self.auswahl = Some(((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)))); self.cursor = b
    }
    pub fn alles_waehlen(&mut self) { self.auswahl = Some(((0, 0), (self.zeilen() - 1, self.kanaele.saturating_sub(1)))) }
    pub fn muster_waehlen(&mut self, p: usize) { if p < self.anzahl_muster() { self.muster = p; self.auswahl = None } }
    pub fn oktave_aendern(&mut self, d: i32) { self.oktave = (self.oktave as i32 + d).clamp(0, 7) as u8 }
    pub fn instrument_aendern(&mut self, d: i32) { self.instrument = (self.instrument as i32 + d).clamp(0, self.anzahl_instrumente().saturating_sub(1) as i32) as usize }
    fn weiter(&mut self) { let n = self.zeilen(); self.cursor.0 = (self.cursor.0 + self.schritt) % n }

    // -- Eingabe --------------------------------------------------------------------------------------------------------
    fn setze_zelle(&mut self, p: usize, r: usize, c: usize, text: &str) {
        if self.ist_play() { if let Some(z) = self.media["steps"].get_mut(r).and_then(|z| z.get_mut(c)) { *z = json!(text) } return }
        if let Some(z) = self.media["patterns"].get_mut(p).and_then(|m| m["rows"].get_mut(r)).and_then(|z| z.get_mut(c)) { *z = json!(text) }
    }
    /// Noteneingabe: Klaviertaste (`halbton` 0..24) in der aktuellen Oktave mit dem gewählten Instrument, danach Zeile weiter
    pub fn note_eingeben(&mut self, halbton: u8) -> Eingabe {
        let note = self.oktave as u32 * 12 + halbton as u32 + 1; if note >= 255 { return Eingabe::Keine }
        self.undo_punkt(); let (r, c, p, i) = (self.cursor.0, self.cursor.1, self.muster, self.instrument);
        self.setze_zelle(p, r, c, &fmt_cell(note as u8, i as u8)); self.weiter(); Eingabe::Note(note as u8, i)
    }
    /// Notenende „===“ setzen
    pub fn notenende(&mut self) { self.undo_punkt(); let (r, c, p) = (self.cursor.0, self.cursor.1, self.muster); self.setze_zelle(p, r, c, "=== --"); self.weiter() }
    /// Zelle (oder Auswahl) leeren; ohne Auswahl rückt der Cursor weiter
    pub fn loeschen(&mut self) {
        self.undo_punkt(); let p = self.muster;
        match self.auswahl { Some(((r0, c0), (r1, c1))) => { for r in r0..=r1 { for c in c0..=c1 { self.setze_zelle(p, r, c, "--- --") } } } None => { let (r, c) = self.cursor; self.setze_zelle(p, r, c, "--- --"); self.weiter() } }
    }
    /// Instrument der Zelle auf das gewählte setzen (nur Zellen mit Note)
    pub fn instrument_anwenden(&mut self) {
        let (p, i) = (self.muster, self.instrument); let ((r0, c0), (r1, c1)) = self.auswahl.unwrap_or((self.cursor, self.cursor)); self.undo_punkt();
        for r in r0..=r1 { for c in c0..=c1 { let (n, _) = parse_cell(&self.zelle(p, r, c)); if n != 0 && n != NOTE_OFF { self.setze_zelle(p, r, c, &fmt_cell(n, i as u8)) } } }
    }

    /// Instrument der Cursorzelle (bzw. aller Noten der Auswahl) um `dx` ändern (Alt+↑/↓); nur Zellen mit Note; liefert (Note, Instrument) der Cursorzelle zum Vorhören
    pub fn zell_instrument_aendern(&mut self, dx: i32) -> Option<(u8, usize)> {
        let (p, n_ins) = (self.muster, self.anzahl_instrumente().max(1)); let ((r0, c0), (r1, c1)) = self.auswahl.unwrap_or((self.cursor, self.cursor)); let mut erster = None;
        let hat_note = (r0..=r1).any(|r| (c0..=c1).any(|c| { let (n, _) = parse_cell(&self.zelle(p, r, c)); n != 0 && n != NOTE_OFF })); if !hat_note { return None }
        self.undo_punkt();
        for r in r0..=r1 { for c in c0..=c1 { let (n, i) = parse_cell(&self.zelle(p, r, c)); if n != 0 && n != NOTE_OFF { let alt = if i == INS_KEEP { self.instrument as i32 } else { i as i32 }; let neu = (alt + dx).clamp(0, n_ins as i32 - 1) as u8; self.setze_zelle(p, r, c, &fmt_cell(n, neu)); if erster.is_none() { erster = Some((n, neu as usize)) } } } }
        erster
    }

    // -- Block ----------------------------------------------------------------------------------------------------------
    fn block(&self) -> Auswahl { self.auswahl.unwrap_or((self.cursor, self.cursor)) }
    pub fn kopieren(&mut self) { let ((r0, c0), (r1, c1)) = self.block(); let p = self.muster; self.zwischen = Some((r0..=r1).map(|r| (c0..=c1).map(|c| self.zelle(p, r, c)).collect()).collect()); self.status = vc_i18n::t!("songdoc-kopiert-p1-p2-zellen", p1 = r1 - r0 + 1, p2 = c1 - c0 + 1) }
    pub fn ausschneiden(&mut self) { self.kopieren(); self.undo_punkt(); let ((r0, c0), (r1, c1)) = self.block(); let p = self.muster; for r in r0..=r1 { for c in c0..=c1 { self.setze_zelle(p, r, c, "--- --") } } }
    /// Block ab dem Cursor einfügen (am Rand abgeschnitten)
    pub fn einfuegen(&mut self) {
        let Some(z) = self.zwischen.clone() else { self.status = vc_i18n::t!("songdoc-zwischenablage-leer-erst-kopieren").into(); return }; self.undo_punkt();
        let (p, (r0, c0), n) = (self.muster, self.cursor, self.zeilen());
        for (dr, zeile) in z.iter().enumerate() { for (dc, t) in zeile.iter().enumerate() { if r0 + dr < n && c0 + dc < self.kanaele { self.setze_zelle(p, r0 + dr, c0 + dc, t) } } }
        self.status = vc_i18n::t!("songdoc-eingefuegt-ab-zeile-r0-kanal-p2", r0 = format!("{:02X}", r0), p2 = c0 + 1)
    }
    /// Noten im Block um `halbtoene` verschieben (Notenende und leere Zellen bleiben)
    pub fn transponieren(&mut self, halbtoene: i32) {
        self.undo_punkt(); let ((r0, c0), (r1, c1)) = self.block(); let p = self.muster;
        for r in r0..=r1 { for c in c0..=c1 { let (n, i) = parse_cell(&self.zelle(p, r, c)); if n != 0 && n != NOTE_OFF { let m = (n as i32 + halbtoene).clamp(1, 254) as u8; self.setze_zelle(p, r, c, &fmt_cell(m, i)) } } }
    }

    // -- Muster und Reihenfolge ---------------------------------------------------------------------------------------
    pub fn muster_neu(&mut self) {
        if self.ist_play() { return }
        self.undo_punkt(); let (n, k) = (self.zeilen(), self.kanaele);
        if let Some(ps) = self.media["patterns"].as_array_mut() { ps.push(json!({ "rows": (0..n).map(|_| vec!["--- --"; k]).collect::<Vec<_>>() })) }
        self.muster = self.anzahl_muster() - 1; self.auswahl = None
    }
    pub fn muster_duplizieren(&mut self) {
        if self.ist_play() { return }
        self.undo_punkt(); let c = self.media["patterns"][self.muster].clone(); if let Some(ps) = self.media["patterns"].as_array_mut() { ps.push(c) } self.muster = self.anzahl_muster() - 1; self.auswahl = None
    }
    /// Muster löschen: Einträge der Reihenfolge, die es nennen, werden entfernt, höhere Nummern rücken nach; das letzte Muster bleibt
    pub fn muster_loeschen(&mut self) {
        if self.ist_play() || self.anzahl_muster() < 2 { return } let del = self.muster; self.undo_punkt();
        if let Some(ps) = self.media["patterns"].as_array_mut() { ps.remove(del); }
        let mut neu: Vec<usize> = self.reihenfolge().into_iter().filter(|&p| p != del).map(|p| if p > del { p - 1 } else { p }).collect(); if neu.is_empty() { neu.push(0) }
        let loop_at = (self.media["loop"].as_u64().unwrap_or(0) as usize).min(neu.len() - 1); self.media["order"] = json!(neu); self.media["loop"] = json!(loop_at);
        self.muster = self.muster.min(self.anzahl_muster() - 1); self.ord_wahl = self.ord_wahl.min(self.reihenfolge().len() - 1); self.auswahl = None
    }
    pub fn ord_waehlen(&mut self, i: usize) { let n = self.reihenfolge().len(); if i < n { self.ord_wahl = i; self.muster_waehlen(self.reihenfolge()[i]) } }
    /// Gewähltes Muster ans Ende der Reihenfolge hängen
    pub fn ord_anhaengen(&mut self) {
        if self.ist_play() { return } self.undo_punkt(); let p = self.muster; if let Some(o) = self.media["order"].as_array_mut() { o.push(json!(p)) } self.ord_wahl = self.reihenfolge().len() - 1 }
    pub fn ord_entfernen(&mut self) {
        if self.ist_play() { return }
        let n = self.reihenfolge().len(); if n < 2 { return } self.undo_punkt(); let i = self.ord_wahl.min(n - 1);
        if let Some(o) = self.media["order"].as_array_mut() { o.remove(i); } let loop_at = (self.media["loop"].as_u64().unwrap_or(0) as usize).min(n - 2); self.media["loop"] = json!(loop_at); self.ord_wahl = i.min(n - 2)
    }
    /// Eintrag der Reihenfolge auf das gewählte Muster setzen
    pub fn ord_setzen(&mut self) {
        if self.ist_play() { return } let (i, p) = (self.ord_wahl, self.muster); if i < self.reihenfolge().len() { self.undo_punkt(); self.media["order"][i] = json!(p) } }
    pub fn ord_verschieben(&mut self, d: i32) {
        if self.ist_play() { return }
        let n = self.reihenfolge().len() as i32; let (i, j) = (self.ord_wahl as i32, self.ord_wahl as i32 + d); if j < 0 || j >= n { return }
        self.undo_punkt(); if let Some(o) = self.media["order"].as_array_mut() { o.swap(i as usize, j as usize) } self.ord_wahl = j as usize
    }
    pub fn schleife_setzen(&mut self, l: usize) {
        if self.ist_play() { return } let n = self.reihenfolge().len(); self.undo_punkt(); self.media["loop"] = json!(l.min(n.saturating_sub(1))) }

    // -- Instrumente -----------------------------------------------------------------------------------------------------
    pub fn instrument_neu(&mut self) {
        let n = self.anzahl_instrumente(); if n >= self.max_instrumente() { return } self.undo_punkt();
        let v = if self.pokey { json!({ "name": format!("neu{n}"), "dist": "pure", "env": [15, 14, 13, 12, 11, 10, 9, 8, 8, 8, 8, 8, 8, 8, 8, 8], "loop": 15 }) } else { json!({ "name": format!("neu{n}"), "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4 }) };
        if let Some(a) = self.media["instruments"].as_array_mut() { a.push(v) } self.ins_wahl = n
    }
    /// Höchstzahl der Instrumente bzw. Hüllkurven (PLAY: ENVELOPE 0–9)
    pub fn max_instrumente(&self) -> usize { if self.ist_play() { 10 } else { 32 } }
    pub fn instrument_duplizieren(&mut self) {
        let n = self.anzahl_instrumente(); if n == 0 || n >= self.max_instrumente() { return } self.undo_punkt(); let v = self.media["instruments"][self.ins_wahl].clone();
        if let Some(a) = self.media["instruments"].as_array_mut() { a.push(v) } self.ins_wahl = n
    }
    /// Wird das Instrument in irgendeinem Muster benutzt?
    pub fn instrument_benutzt(&self, i: usize) -> bool {
        self.media["patterns"].as_array().is_some_and(|ps| ps.iter().any(|p| p["rows"].as_array().is_some_and(|rs| rs.iter().any(|r| r.as_array().is_some_and(|cs| cs.iter().any(|c| { let (n, ins) = parse_cell(c.as_str().unwrap_or("")); n != 0 && n != NOTE_OFF && ins as usize == i }))))))
    }
    /// Löschen nur für das letzte, unbenutzte Instrument (sonst würden Verweise im Raster falsch)
    pub fn instrument_loeschen(&mut self) -> bool {
        let n = self.anzahl_instrumente(); if n < 2 || self.ins_wahl + 1 != n || self.instrument_benutzt(self.ins_wahl) { return false }
        self.undo_punkt(); if let Some(a) = self.media["instruments"].as_array_mut() { a.pop(); } self.ins_wahl = n - 2; self.instrument = self.instrument.min(n - 2); true
    }
    /// Wert eines Instrument-Feldes setzen; `erster` = Rückgängig-Punkt (Regler ziehen = ein Schritt)
    pub fn instrument_wert(&mut self, key: &str, wert: Value, erster: bool) { if erster { self.undo_punkt() } let i = self.ins_wahl; if !self.media["instruments"][i].is_null() { self.media["instruments"][i][key] = wert } }
    /// Lautstärke des Bildes `k` der POKEY-Hüllkurve (0–15)
    pub fn huellkurve_setzen(&mut self, k: usize, v: u8, erster: bool) {
        if erster { self.undo_punkt() } let i = self.ins_wahl; let mut env: Vec<u64> = self.media["instruments"][i]["env"].as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0)).collect()).unwrap_or_else(|| vec![15; 16]); env.resize(16, 0);
        if k < 16 { env[k] = v.min(15) as u64 } self.media["instruments"][i]["env"] = json!(env)
    }
    pub fn instrument_waehlen(&mut self, i: usize) { if i < self.anzahl_instrumente() { self.ins_wahl = i; self.instrument = i } }
    pub fn stumm_umschalten(&mut self, c: usize) { if let Some(x) = self.stumm.get_mut(c) { *x = !*x } }
    pub fn solo_umschalten(&mut self, c: usize) { if let Some(x) = self.solo.get_mut(c) { *x = !*x } }

    // -- Struktur -------------------------------------------------------------------------------------------------------
    pub fn zeilen_setzen(&mut self, n: usize) {
        let n = n.clamp(1, 128); self.undo_punkt(); let k = self.kanaele;
        if self.ist_play() { if let Some(ss) = self.media["steps"].as_array_mut() { while ss.len() < n { ss.push(json!(vec!["--- --"; k])) } ss.truncate(n) } self.media["rows"] = json!(n); self.nach_aenderung(); return }
        if let Some(ps) = self.media["patterns"].as_array_mut() { for p in ps { if let Some(rows) = p["rows"].as_array_mut() { while rows.len() < n { rows.push(json!(vec!["--- --"; k])) } rows.truncate(n) } } }
        self.media["rows"] = json!(n); self.nach_aenderung()
    }
    pub fn tempo_setzen(&mut self, t: u64) { self.undo_punkt(); if self.ist_play() { self.media["tempo"] = json!(t.clamp(1, 255)) } else { self.media["speed"] = json!(t.clamp(1, 31)) } }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
    fn kopie(tag: &str, bsp: &str) -> (PathBuf, String) {
        let dir = std::env::temp_dir().join(format!("vc-songdoc-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("music")).unwrap();
        let rel = format!("music/{}", bsp.rsplit('/').next().unwrap()); std::fs::copy(repo().join("examples").join(bsp), dir.join(&rel)).unwrap(); (dir, rel)
    }

    #[test]
    fn klaviertasten_wie_am_tracker() {
        assert_eq!((klaviertaste('z'), klaviertaste('M'), klaviertaste('q'), klaviertaste('i'), klaviertaste('2'), klaviertaste('k')), (Some(0), Some(11), Some(12), Some(24), Some(13), None));
    }

    #[test]
    fn oeffnen_eingeben_navigieren_speichern_c64_und_atari_mega65() {
        let data = repo().join("data");
        for (bsp, kan, chip) in [("c64/music_demo/music/tune.vcsong", 3, "sid"), ("atari/music_demo/music/tune_pokey.vcsong", 4, "pokey"), ("mega65/music_demo/music/tune4.vcsong", 12, "sid")] {
            let (dir, rel) = kopie("a", bsp); let mut s = SongDoc::oeffnen(&dir, &rel, &data).unwrap();
            assert_eq!((s.kanaele, s.chip.as_str(), s.zeilen()), (kan, chip, 32), "{bsp}"); assert!(s.anzahl_muster() >= 1 && !s.reihenfolge().is_empty() && s.anzahl_instrumente() >= 1 && !s.dirty());
            // Note eingeben: C-3 (Taste Z in Oktave 3) mit Instrument 0, Cursor rückt eine Zeile weiter
            s.setze_cursor(4, 0); s.oktave = 3; s.instrument = 0; let e = s.note_eingeben(klaviertaste('z').unwrap()); assert!(matches!(e, Eingabe::Note(37, 0)), "C-3 = 37"); assert_eq!((s.zelle(0, 4, 0).as_str(), s.cursor), ("C-3 00", (5, 0))); assert!(s.dirty());
            s.note_eingeben(klaviertaste('2').unwrap()); assert_eq!(s.zelle(0, 5, 0), "C#4 00"); s.notenende(); assert_eq!(s.zelle(0, 6, 0), "=== --");
            s.setze_cursor(4, 0); s.loeschen(); assert_eq!((s.zelle(0, 4, 0).as_str(), s.cursor.0), ("--- --", 5), "Löschen rückt vor");
            s.rueckgaengig(); assert_eq!(s.zelle(0, 4, 0), "C-3 00"); s.wiederholen(); assert_eq!(s.zelle(0, 4, 0), "--- --");
            // Navigation mit Umlauf, Seiten, Auswahl per Umschalt
            s.setze_cursor(31, kan - 1); s.bewege(1, 1, false); assert_eq!(s.cursor, (0, 0), "Umlauf"); s.bewege(-1, 0, false); assert_eq!(s.cursor.0, 31);
            s.setze_cursor(2, 1.min(kan - 1)); s.bewege(2, 0, true); assert_eq!(s.auswahl, Some(((2, 1.min(kan - 1)), (4, 1.min(kan - 1))))); s.bewege(0, 0, false); assert!(s.auswahl.is_none());
            s.speichern().unwrap(); assert!(!s.dirty()); let s2 = SongDoc::oeffnen(&dir, &rel, &data).unwrap(); assert_eq!(s2.zelle(0, 5, 0), "C#4 00");
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn muster_reihenfolge_und_instrumente_bearbeiten() {
        let data = repo().join("data"); let (dir, rel) = kopie("m", "c64/music_demo/music/tune.vcsong"); let mut s = SongDoc::oeffnen(&dir, &rel, &data).unwrap();
        assert_eq!((s.anzahl_muster(), s.reihenfolge()), (2, vec![0, 1, 0, 1]));
        // Muster neu/duplizieren; Reihenfolge anhängen, setzen, verschieben, entfernen, Schleife
        s.muster_neu(); assert_eq!((s.anzahl_muster(), s.muster, s.zelle(2, 5, 1).as_str()), (3, 2, "--- --")); s.muster_waehlen(0); s.muster_duplizieren(); assert_eq!((s.anzahl_muster(), s.zelle(3, 0, 0)), (4, s.zelle(0, 0, 0)));
        s.muster_waehlen(2); s.ord_anhaengen(); assert_eq!(s.reihenfolge(), vec![0, 1, 0, 1, 2]); s.ord_waehlen(0); s.muster_waehlen(3); s.ord_setzen(); assert_eq!(s.reihenfolge()[0], 3); s.ord_verschieben(1); assert_eq!((s.reihenfolge()[..2].to_vec(), s.ord_wahl), (vec![1, 3], 1));
        s.schleife_setzen(2); assert_eq!(s.media["loop"], 2); s.ord_waehlen(4); s.ord_entfernen(); assert_eq!(s.reihenfolge().len(), 4);
        // Muster löschen: Verweise verschwinden, höhere Nummern rücken nach
        s.muster_waehlen(1); s.muster_loeschen(); assert_eq!(s.anzahl_muster(), 3); assert!(s.reihenfolge().iter().all(|&p| p < 3) && !s.reihenfolge().is_empty(), "{:?}", s.reihenfolge()); assert_eq!(s.reihenfolge(), vec![2, 0]); s.rueckgaengig(); assert_eq!(s.anzahl_muster(), 4);
        // Instrumente: neu, Wert setzen (ein Schritt), duplizieren, löschen nur letztes unbenutztes
        let n0 = s.anzahl_instrumente(); s.instrument_neu(); assert_eq!((s.anzahl_instrumente(), s.ins_wahl), (n0 + 1, n0)); s.instrument_wert("a", json!(5), true); s.instrument_wert("a", json!(6), false); assert_eq!(s.media["instruments"][n0]["a"], 6); let u = s.undo.len(); s.instrument_wert("s", json!(3), true); assert_eq!(s.undo.len(), u + 1);
        s.instrument_duplizieren(); assert_eq!(s.anzahl_instrumente(), n0 + 2); assert!(s.instrument_loeschen()); assert_eq!(s.anzahl_instrumente(), n0 + 1); s.instrument_waehlen(0); assert!(!s.instrument_loeschen(), "nur das letzte lässt sich löschen"); assert!(s.instrument_benutzt(0));
        s.stumm_umschalten(1); s.solo_umschalten(0); assert_eq!((s.stumm.clone(), s.solo.clone()), (vec![false, true, false], vec![true, false, false]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn play_tracker_wie_ein_song_mit_einem_muster() {
        let data = repo().join("data"); let dir = std::env::temp_dir().join(format!("vc-songdoc-play-{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("music")).unwrap();
        std::fs::copy(repo().join("examples/mega65/play_demo/music/tune_play.vcplay"), dir.join("music/tune_play.vcplay")).unwrap(); let rel = "music/tune_play.vcplay";
        let mut s = SongDoc::oeffnen(&dir, rel, &data).unwrap(); assert!(s.ist_play() && !s.pokey); assert_eq!((s.kanaele, s.zeilen(), s.anzahl_muster(), s.reihenfolge(), s.tempo(), s.lautstaerke()), (6, 64, 1, vec![0], 8, 8)); assert_eq!(s.zelle(0, 0, 0), "C-4 00");
        s.setze_cursor(1, 2); s.oktave = 3; s.note_eingeben(klaviertaste('z').unwrap()); assert_eq!(s.zelle(0, 1, 2), "C-3 00"); assert_eq!(s.media["steps"][1][2], "C-3 00", "schreibt in steps, nicht in patterns"); assert!(s.media.get("patterns").is_none());
        s.muster_neu(); s.ord_anhaengen(); assert_eq!((s.anzahl_muster(), s.reihenfolge()), (1, vec![0]), "es gibt nur ein Muster");
        s.zeilen_setzen(16); assert_eq!((s.zeilen(), s.media["steps"].as_array().unwrap().len()), (16, 16)); s.zeilen_setzen(70); assert_eq!(s.zeilen(), 70); s.tempo_setzen(200); assert_eq!(s.media["tempo"], 200); s.lautstaerke_setzen(30); assert_eq!(s.lautstaerke(), 15);
        let d = s.song_daten(false); assert_eq!((d.channels, d.rows, d.order.clone(), d.row_secs), (6, 70, vec![0], 1.0 / 200.0)); assert_eq!(d.patterns[0][0][0], (49, 0));
        let n0 = s.anzahl_instrumente(); for _ in 0..12 { s.instrument_neu() } assert_eq!(s.anzahl_instrumente(), 10.max(n0), "höchstens 10 Hüllkurven (ENVELOPE 0–9)");
        s.speichern().unwrap(); let s2 = SongDoc::oeffnen(&dir, rel, &data).unwrap(); assert_eq!((s2.zelle(0, 1, 2).as_str(), s2.tempo()), ("C-3 00", 200)); let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pokey_huellkurve() {
        let data = repo().join("data"); let (dir, rel) = kopie("p", "atari/music_demo/music/tune_pokey.vcsong"); let mut s = SongDoc::oeffnen(&dir, &rel, &data).unwrap(); assert!(s.pokey);
        s.instrument_waehlen(0); s.huellkurve_setzen(3, 9, true); s.huellkurve_setzen(4, 20, false); let env: Vec<u64> = s.media["instruments"][0]["env"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).collect(); assert_eq!((env.len(), env[3], env[4]), (16, 9, 15), "auf 15 begrenzt");
        s.instrument_wert("dist", json!("buzz"), true); assert_eq!(s.instrumente()[0], Instr::Pokey { dist: "buzz".into(), env: env.iter().map(|x| *x as u8).collect(), loop_at: s.media["instruments"][0]["loop"].as_u64().unwrap_or(15) as usize });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn block_kopieren_einfuegen_transponieren_zeilen_und_sequenzerdaten() {
        let data = repo().join("data"); let (dir, rel) = kopie("b", "c64/music_demo/music/tune.vcsong"); let mut s = SongDoc::oeffnen(&dir, &rel, &data).unwrap();
        s.setze_cursor(0, 0); s.oktave = 2; s.note_eingeben(0); s.note_eingeben(4); s.note_eingeben(7); // C-2 E-2 G-2 in Kanal 0, Zeilen 0–2
        s.waehle((0, 0), (2, 0)); s.kopieren(); assert!(s.status.starts_with("Kopiert (3×1")); s.setze_cursor(8, 1); s.einfuegen(); assert_eq!((s.zelle(0, 8, 1).as_str(), s.zelle(0, 9, 1).as_str(), s.zelle(0, 10, 1).as_str()), ("C-2 00", "E-2 00", "G-2 00"));
        s.waehle((8, 1), (10, 1)); s.transponieren(12); assert_eq!(s.zelle(0, 8, 1), "C-3 00"); s.transponieren(-24); assert_eq!(s.zelle(0, 8, 1), "C-1 00"); s.ausschneiden(); assert_eq!(s.zelle(0, 9, 1), "--- --");
        s.setze_cursor(31, 0); s.einfuegen(); assert_eq!(s.zelle(0, 31, 0), "C-1 00", "am Rand abgeschnitten: nur die erste Zeile passt"); s.instrument = 1; s.waehle((31, 0), (31, 0)); s.instrument_anwenden(); assert_eq!(s.zelle(0, 31, 0), "C-1 01");
        s.zeilen_setzen(16); assert_eq!((s.zeilen(), s.media["patterns"][0]["rows"].as_array().unwrap().len()), (16, 16)); assert_eq!(s.cursor.0, 15); s.zeilen_setzen(48); assert_eq!(s.zelle(0, 40, 0), "--- --"); s.tempo_setzen(4); assert_eq!(s.tempo(), 4);
        let d = s.song_daten(false); assert_eq!((d.channels, d.rows, d.row_secs), (3, 48, 4.0 / 50.0)); assert_eq!(d.patterns[0][0][0], (25, 0)); assert_eq!(s.song_daten(true).order, vec![0]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
