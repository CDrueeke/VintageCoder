// SPDX-License-Identifier: GPL-3.0-or-later
//! Disk-Editor (E2): `DiskDoc` (vc-ide-model) als Dokument im `DocStore`. Die Ansicht (`view`) ist das Anzeige-DTO für Verzeichnis, Belegungskarte und
//! Sektor-Inspektor; Absichten (`apply`) wählen Datei und Sektor, folgen der Kette und blenden sie ein oder aus. Zahlen und Namen kommen aus dem Kern.
use crate::docstore::{Dokument, Fehler};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_disk::{Addr, Cell, Format};
use vc_ide_model::diskdoc::{self, DiskDoc};

pub struct DiskDok { pub doc: DiskDoc, /** Meldungen der letzten Übernahme ins Projekt (Erfolg, Text) */ pub uebernahme: Option<Vec<(bool, String)>> }

impl DiskDok {
    pub fn doc_view(&self) -> Value { Dokument::view(self) }
    pub fn oeffnen(path: &Path, data: Option<PathBuf>) -> Result<DiskDok, Fehler> {
        let doc = DiskDoc::oeffnen(path, data);
        if let Err(e) = &doc.disk { return Err(Fehler::mit("disk-unlesbar", json!({ "detail": e, "path": path.to_string_lossy() }))); }
        Ok(DiskDok { doc, uebernahme: None })
    }
}

fn zelle(c: Cell) -> i64 { match c { Cell::Free => -1, Cell::System => -2, Cell::Dir => -3, Cell::Orphan => -4, Cell::Conflict => -5, Cell::File(i) => i as i64 } }
fn adr(a: Addr) -> Value { json!([a.track, a.sector]) }
fn format_label(f: Format) -> &'static str { f.label() }

/// Anzeige-DTO des Dokuments
pub fn ansicht(d: &DiskDoc) -> Value {
    let Ok(disk) = &d.disk else { return json!({ "error": true }) };
    let fmt = disk.format(); let atari = fmt.is_atari();
    let files: Vec<Value> = d.files.iter().enumerate().map(|(i, f)| {
        let e = d.img.as_ref().and_then(|im| im.entries.get(i));
        json!({ "i": i, "name": diskdoc::name_text(atari, &f.name), "kind": f.kind, "blocks": f.blocks, "start": diskdoc::addr_text(fmt, f.start), "locked": f.locked, "auto": f.autostart,
                "size": e.map(|e| if e.data.is_ok() { diskdoc::size_text(e.size()) } else { String::new() }), "bad": e.is_some_and(|e| e.data.is_err()),
                "checked": d.checked.get(i).copied().unwrap_or(false), "basic": d.basic.get(i).map_or(false, |b| b.is_some()) })
    }).collect();
    let cells: Vec<Value> = d.an.cells.iter().map(|r| Value::Array(r.iter().map(|c| json!(zelle(*c))).collect())).collect();
    let total: usize = d.an.cells.iter().map(|r| r.len()).sum();
    let sektor = disk.sector(d.cur);
    let hex: Value = d.sektor_zeilen(8).map(|z| Value::Array(z.into_iter().map(|h| json!({ "off": h.versatz, "bytes": h.bytes.iter().map(|(b, p)| json!([b, p])).collect::<Vec<_>>(), "text": h.text })).collect())).unwrap_or(Value::Null);
    let bytes_im_sektor = sektor.as_ref().map(|s| if atari { s[127] as usize } else if s[0] == 0 { s[1] as usize } else { 254 });
    let kette: Vec<Value> = d.kette().into_iter().map(adr).collect();
    json!({
        "format": format_label(fmt), "atari": atari, "title": d.titel(), "path": d.path.as_ref().map(|p| p.to_string_lossy()), "dirty": d.dirty, "geometry": diskdoc::geometry_text(disk),
        "size": d.img.as_ref().map(|_| diskdoc::size_text(disk.to_bytes().len())),
        "files": files, "cells": cells, "free": disk.free_blocks(), "used": d.an.used, "total": total, "interleave": d.an.interleave,
        "problems": d.an.problems, "report": d.report, "selFile": d.sel_file, "chainOn": d.kette_an, "chain": kette, "chainPos": d.kettenposition(),
        "cur": adr(d.cur), "curText": diskdoc::addr_text(fmt, d.cur), "title2": d.sektor_titel(), "owner": d.besitzer(), "pointer": d.zeiger_hinweis(), "hex": hex,
        "next": d.folge().map(|a| json!({ "addr": adr(a), "text": diskdoc::addr_text(fmt, a) })), "bytesInSector": bytes_im_sektor, "canBack": !d.history.is_empty(),
        "checkedCount": d.anzahl_vorgemerkt(), "canUndo": !d.undo.is_empty(), "canRedo": !d.redo.is_empty(), "canOrder": disk.has_filesystem() && !(atari && disk.sector_size() != 128) && d.files.len() > 1, "note": d.note.as_ref().map(|(ok, t)| json!({ "ok": ok, "text": t })), "fs": disk.has_filesystem(),
        "opts": { "dir": d.opts.dir, "basic": d.opts.basic, "raw": d.opts.raw, "asm": d.opts.asm, "dirBad": d.zielordner_ungueltig() },
    })
}

impl Dokument for DiskDok {
    fn art(&self) -> &'static str { "disk" }
    fn view(&self) -> Value { let mut v = ansicht(&self.doc); v["importResult"] = self.uebernahme.as_ref().map_or(Value::Null, |m| json!(m.iter().map(|(ok, t)| json!({ "ok": ok, "text": t })).collect::<Vec<_>>())); v }
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
        self.doc.note = None; if op["kind"] != "import" { self.uebernahme = None; }
        let n = |k: &str| op[k].as_u64().map(|x| x as usize).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        match op["kind"].as_str().unwrap_or("") {
            "file_select" => { let i = n("i")?; if i >= self.doc.files.len() { return Err(Fehler::neu("op-ungueltig")); } self.doc.datei_waehlen(i); }
            "sector_select" => { let a = Addr { track: n("track")?, sector: n("sector")? }; if self.doc.an.cell(a).is_none() { return Err(Fehler::neu("op-ungueltig")); } self.doc.sektor_waehlen(a); }
            "follow" => self.doc.folgen(),
            "back" => self.doc.zurueck(),
            "import" => {
                if self.doc.zielordner_ungueltig() { return Err(Fehler::neu("disk-zielordner")); }
                let root = PathBuf::from(op["root"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "root" })))?);
                if self.doc.anzahl_vorgemerkt() == 0 { return Err(Fehler::neu("disk-nichts-vorgemerkt")); }
                self.uebernahme = Some(self.doc.uebernehmen(&root));
            }
            "insert" => {
                // Standard-Emulator des Systems mit dieser Diskette (Laufwerk 8); erst gespeichert, dann eingelegt
                let Some(p) = self.doc.path.clone() else { return Err(Fehler::neu("disk-erst-speichern")) };
                if self.doc.dirty { return Err(Fehler::neu("disk-erst-speichern")); }
                let fmt = self.doc.format().ok_or_else(|| Fehler::neu("op-ungueltig"))?; let ziel = diskdoc::target_for(fmt);
                let variant = op["root"].as_str().map(|r| vc_ide::launch::atari_variant(Path::new(r))).unwrap_or_else(|| "pal".into());
                let r = if crate::bau::gesperrt() { Ok(format!("{} {}", ziel, p.display())) } else { vc_ide::launch::insert_disk(&p, ziel, &variant) };
                self.doc.note = Some(match r { Ok(c) => (true, vc_i18n::t!("disk-eingelegt", c = &c).to_string()), Err(e) => (false, e) });
            }
            "file_move" => { let (von, nach) = (n("from")?, n("to")?); self.doc.datei_verschieben(von, nach).map_err(|e| Fehler::mit("disk-verschieben", json!({ "detail": e })))?; }
            "undo" => { if !self.doc.rueckgaengig() { return Err(Fehler::neu("undo-leer")); } }
            "redo" => { if !self.doc.wiederholen() { return Err(Fehler::neu("redo-leer")); } }
            "validate" => self.doc.validieren(),
            "check" => { let i = n("i")?; if i >= self.doc.files.len() { return Err(Fehler::neu("op-ungueltig")); } self.doc.vormerken(i, op["on"].as_bool().unwrap_or(true)); }
            "check_all" => self.doc.alle_vormerken(),
            "check_none" => self.doc.keine_vormerken(),
            "check_prg" => self.doc.nur_programme(),
            "check_basic" => self.doc.nur_basic(),
            "opts" => {
                if let Some(x) = op["dir"].as_str() { self.doc.opts.dir = x.to_string(); }
                for (k, f) in [("basic", 0), ("raw", 1), ("asm", 2)] { if let Some(b) = op[k].as_bool() { match f { 0 => self.doc.opts.basic = b, 1 => self.doc.opts.raw = b, _ => self.doc.opts.asm = b } } }
            }
            "add_files" => { let ps: Vec<PathBuf> = op["paths"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(PathBuf::from)).collect()).unwrap_or_default(); if ps.is_empty() { return Err(Fehler::neu("op-ungueltig")); } self.doc.dateien_ablegen(&ps); }
            "save" => { if self.doc.path.is_none() { return Err(Fehler::neu("disk-ohne-pfad")); } self.doc.speichern(); }
            "save_as" => { let p = op["path"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "path" })))?; self.doc.speichern_unter(PathBuf::from(p)); }
            "chain" => self.doc.kette_an = op["on"].as_bool().unwrap_or(!self.doc.kette_an),
            _ => return Err(Fehler::neu("op-ungueltig")),
        }
        Ok(self.view())
    }
    fn undo(&mut self) -> Result<Value, Fehler> { if !self.doc.rueckgaengig() { return Err(Fehler::neu("undo-leer")); } Ok(self.view()) }
    fn redo(&mut self) -> Result<Value, Fehler> { if !self.doc.wiederholen() { return Err(Fehler::neu("redo-leer")); } Ok(self.view()) }
    fn save(&mut self) -> Result<(), Fehler> { self.doc.speichern(); Ok(()) }
    fn undo_tiefe(&self) -> usize { self.doc.undo.len() }
    fn ungesichert(&self) -> bool { self.doc.dirty }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn beispiel() -> Option<DiskDok> {
        // ein im Quellbaum liegendes Abbild; fehlt keines, bleibt der Test still (die Beispiele liegen nicht überall)
        let wurzel = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = std::fs::read_dir(wurzel.join("examples")).ok()?.flatten().flat_map(|s| std::fs::read_dir(s.path()).into_iter().flatten().flatten()).flat_map(|p| std::fs::read_dir(p.path()).into_iter().flatten().flatten())
            .map(|f| f.path()).find(|f| f.extension().is_some_and(|e| e.eq_ignore_ascii_case("d64") || e.eq_ignore_ascii_case("d81") || e.eq_ignore_ascii_case("atr")))?;
        DiskDok::oeffnen(&p, Some(wurzel.join("data"))).ok()
    }
    #[test] fn neue_diskette_hat_ansicht_und_sektor_aktionen() {
        let doc = DiskDoc::neu(Format::D64, "TEST", "01", None).unwrap();
        let mut d = DiskDok { doc, uebernahme: None };
        let v = d.view();
        assert_eq!(v["format"], "D64"); assert_eq!(v["cells"].as_array().unwrap().len(), 35); assert_eq!(v["cur"], json!([18, 0]));
        assert!(v["hex"].as_array().unwrap().len() == 32, "256 Bytes in Zeilen zu 8");
        let v = d.apply(&json!({ "kind": "sector_select", "track": 1, "sector": 0 })).unwrap(); assert_eq!(v["cur"], json!([1, 0])); assert_eq!(v["canBack"], true);
        let v = d.apply(&json!({ "kind": "back" })).unwrap(); assert_eq!(v["cur"], json!([18, 0]));
        assert_eq!(d.apply(&json!({ "kind": "gibtsnicht" })).unwrap_err().code, "op-ungueltig");
        assert_eq!(d.apply(&json!({ "kind": "sector_select", "track": 99, "sector": 0 })).unwrap_err().code, "op-ungueltig");
        assert_eq!(d.apply(&json!({ "kind": "file_select", "i": 5 })).unwrap_err().code, "op-ungueltig");
    }
    #[test] fn beispielabbild_zeigt_verzeichnis_und_kette() {
        let Some(mut d) = beispiel() else { return };
        let v = d.view();
        if v["files"].as_array().unwrap().is_empty() { return; }
        let v = d.apply(&json!({ "kind": "file_select", "i": 0 })).unwrap();
        assert!(!v["chain"].as_array().unwrap().is_empty(), "die Kette der gewählten Datei ist da");
        assert!(v["chainPos"].is_array());
    }
    #[test] fn pruefen_vormerken_ablegen_und_speichern() {
        let tmp = std::env::temp_dir().join(format!("vc_disk_e22_{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        let mut d = DiskDok { doc: DiskDoc::neu(Format::D64, "TEST", "01", Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"))).unwrap(), uebernahme: None };
        std::fs::write(tmp.join("eins.prg"), [0x01u8, 0x08, 0x60, 0x60, 0x60]).unwrap();
        let v = d.apply(&json!({ "kind": "add_files", "paths": [tmp.join("eins.prg").to_string_lossy()] })).unwrap();
        assert_eq!(v["files"].as_array().unwrap().len(), 1); assert_eq!(v["dirty"], true); assert_eq!(v["note"]["ok"], true);
        let v = d.apply(&json!({ "kind": "validate" })).unwrap(); assert_eq!(v["report"].as_array().map(|r| r.len()), Some(0), "frisch beschriebene Diskette ist in Ordnung: {:?}", v["report"]);
        let v = d.apply(&json!({ "kind": "check_prg" })).unwrap(); assert_eq!(v["checkedCount"], 1);
        let v = d.apply(&json!({ "kind": "check", "i": 0, "on": false })).unwrap(); assert_eq!(v["checkedCount"], 0);
        let v = d.apply(&json!({ "kind": "check_all" })).unwrap(); assert_eq!(v["checkedCount"], 1);
        let v = d.apply(&json!({ "kind": "opts", "dir": "../raus", "asm": true })).unwrap(); assert_eq!(v["opts"]["dirBad"], true); assert_eq!(v["opts"]["asm"], true);
        d.apply(&json!({ "kind": "opts", "dir": "import", "asm": false })).unwrap();
        assert_eq!(d.apply(&json!({ "kind": "save" })).unwrap_err().code, "disk-ohne-pfad");
        let ziel = tmp.join("neu.d64"); let v = d.apply(&json!({ "kind": "save_as", "path": ziel.to_string_lossy() })).unwrap();
        assert_eq!(v["dirty"], false); assert!(ziel.exists());
        let wieder = DiskDok::oeffnen(&ziel, None).unwrap(); assert_eq!(wieder.view()["files"].as_array().unwrap().len(), 1, "gespeichertes Abbild enthält die Datei");
        let v = d.apply(&json!({ "kind": "import", "root": tmp.to_string_lossy() })).unwrap(); let m: Vec<(bool, String)> = v["importResult"].as_array().unwrap().iter().map(|x| (x["ok"].as_bool().unwrap(), x["text"].as_str().unwrap().to_string())).collect(); assert!(m.iter().any(|(ok, _)| *ok), "{:?}", m); assert!(tmp.join("import").is_dir());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn dateien_umordnen_ist_ein_rueckgaengig_schritt() {
        let tmp = std::env::temp_dir().join(format!("vc_disk_ord_{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        let mut d = DiskDok { doc: DiskDoc::neu(Format::D64, "T", "01", None).unwrap(), uebernahme: None };
        for n in ["eins", "zwei", "drei"] { std::fs::write(tmp.join(format!("{n}.prg")), [0x01u8, 0x08, 0x60]).unwrap(); d.apply(&json!({ "kind": "add_files", "paths": [tmp.join(format!("{n}.prg")).to_string_lossy()] })).unwrap(); }
        let namen = |v: &Value| v["files"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap().to_string()).collect::<Vec<_>>();
        let v = d.view(); assert_eq!(namen(&v), ["EINS", "ZWEI", "DREI"]); assert_eq!(v["canUndo"], true, "Ablegen ist rückgängig machbar"); assert_eq!(v["canOrder"], true);
        d.apply(&json!({ "kind": "file_select", "i": 0 })).unwrap(); d.apply(&json!({ "kind": "check", "i": 0, "on": true })).unwrap();
        let tiefe = d.undo_tiefe();
        let v = d.apply(&json!({ "kind": "file_move", "from": 0, "to": 2 })).unwrap();
        assert_eq!(namen(&v), ["ZWEI", "DREI", "EINS"]); assert_eq!(v["selFile"], 2, "die gewählte Datei wandert mit"); assert_eq!(v["files"][2]["checked"], true, "die Vormerkung wandert mit"); assert_eq!(v["dirty"], true);
        assert_eq!(d.undo_tiefe(), tiefe + 1, "ein Zug = ein Rückgängig-Schritt");
        let v = d.undo().unwrap(); assert_eq!(namen(&v), ["EINS", "ZWEI", "DREI"]); assert_eq!(d.undo_tiefe(), tiefe);
        let v = d.redo().unwrap(); assert_eq!(namen(&v), ["ZWEI", "DREI", "EINS"]);
        assert_eq!(d.apply(&json!({ "kind": "file_move", "from": 0, "to": 7 })).unwrap_err().code, "disk-verschieben");
        // Speichern und Rückgängig: nach dem Speichern ist die Diskette sauber, Rückgängig macht sie wieder „geändert“
        let ziel = tmp.join("o.d64"); let v = d.apply(&json!({ "kind": "save_as", "path": ziel.to_string_lossy() })).unwrap(); assert_eq!(v["dirty"], false);
        let v = d.undo().unwrap(); assert_eq!(v["dirty"], true); let v = d.redo().unwrap(); assert_eq!(v["dirty"], false, "zurück auf dem gespeicherten Stand");
        assert_eq!(namen(&DiskDok::oeffnen(&ziel, None).unwrap().view()), ["ZWEI", "DREI", "EINS"]);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn einlegen_erst_nach_dem_speichern() {
        let mut d = DiskDok { doc: DiskDoc::neu(Format::D64, "T", "01", None).unwrap(), uebernahme: None };
        assert_eq!(d.apply(&json!({ "kind": "insert" })).unwrap_err().code, "disk-erst-speichern", "neu und ohne Pfad");
        let ziel = std::env::temp_dir().join(format!("vc_disk_ins_{}.d64", std::process::id()));
        d.apply(&json!({ "kind": "save_as", "path": ziel.to_string_lossy() })).unwrap();
        let v = d.apply(&json!({ "kind": "insert" })).unwrap(); assert_eq!(v["note"]["ok"], true, "im Test wird nichts gestartet");
        d.doc.dirty = true; assert_eq!(d.apply(&json!({ "kind": "insert" })).unwrap_err().code, "disk-erst-speichern", "ungespeichert");
        let _ = std::fs::remove_file(&ziel);
    }
    #[test] fn kaputtes_abbild_meldet_fehlercode() {
        let p = std::env::temp_dir().join(format!("vc_disk_kaputt_{}.d64", std::process::id())); std::fs::write(&p, b"zu kurz").unwrap();
        assert_eq!(DiskDok::oeffnen(&p, None).err().map(|e| e.code), Some("disk-unlesbar".to_string())); let _ = std::fs::remove_file(&p);
    }
}
