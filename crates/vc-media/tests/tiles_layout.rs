// SPDX-License-Identifier: GPL-3.0-or-later
//! Goldene Tests der Kachel-Export-Layouts (K2): planar/interleaved, Karte zeilen-/spaltenweise, 8-/16-Bit-Index. Je Layout werden die erzeugten Bytes
//! mit festen Werten verglichen; die Vorgabe (planar, zeilenweise, 8 Bit) muss weiter exakt so aussehen wie vor K2.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_media::{project, tiles::TilesLayout};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-tlay-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("graphics")).unwrap(); d }
fn write(p: &Path, v: &Value) { std::fs::write(p, project::format_media(v)).unwrap(); }
fn charset(dir: &Path) {
    let frames: Vec<Value> = (0..256).map(|i| json!({ "rows": (0..8).map(|y| if i > 0 && i < 6 && y == 0 { format!("{:08b}", 1u32 << (i - 1)) } else { "00000000".to_string() }).collect::<Vec<_>>() })).collect();
    write(&dir.join("graphics/a.vcchar"), &json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": "a", "colors": {}, "frames": frames }));
}
/// 3 Kacheln 2×2 (leer, 1-2-3-4, 4-3-2-1), Farbe je Kachel 1/5/6, Eigenschaft „solid“ an Kachel 1; Bank mit einem Screen 6×4 Zellen, Karte 3×2 Kacheln
fn projekt(name: &str, bank_extra: Value) -> PathBuf {
    let dir = tmp(name); charset(&dir);
    write(&dir.join("graphics/t.vctiles"), &json!({ "format": 1, "kind": "tileset", "profile": "c64.tileset", "name": "t", "size": { "w": 2, "h": 2 }, "colorMode": "tile", "charsets": ["graphics/a.vcchar"], "propDefs": [{ "name": "solid", "bit": 0 }],
        "tiles": [{ "name": "leer", "cells": [0, 0, 0, 0], "col": 1 }, { "name": "a", "cells": [1, 2, 3, 4], "col": 5, "props": ["solid"] }, { "name": "b", "cells": [4, 3, 2, 1], "col": 6 }] }));
    let mut bank = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "lv", "tilesets": ["graphics/t.vctiles"],
        "screens": [{ "name": "eins", "w": 6, "h": 4, "colors": { "border": 0, "bg": 0 }, "tiles": { "set": 0, "origin": [0, 0], "grid": [[0, 1, 2], [2, 1, 0]] } }] });
    for (k, x) in bank_extra.as_object().unwrap() { bank[k] = x.clone(); }
    write(&dir.join("graphics/lv.vcscreen"), &bank); dir
}
fn bin(dir: &Path, n: &str) -> Vec<u8> { std::fs::read(dir.join("build/export").join(n)).unwrap_or_else(|e| panic!("{n}: {e}")) }
fn exportiere(dir: &Path, rel: &str, label: &str, lay: Option<TilesLayout>) -> Value { project::export_mit(dir, &data(), rel, "bin", Some(label), None, 0, 0, lay).unwrap() }

#[test]
fn vorgabe_ist_planar_zeilenweise_8_bit_wie_vor_k2() {
    let dir = projekt("vorgabe", json!({}));
    exportiere(&dir, "graphics/t.vctiles", "t", None); exportiere(&dir, "graphics/lv.vcscreen", "lv", None);
    assert_eq!((bin(&dir, "t.c0.bin"), bin(&dir, "t.c1.bin"), bin(&dir, "t.c2.bin"), bin(&dir, "t.c3.bin")), (vec![0, 1, 4], vec![0, 2, 3], vec![0, 3, 2], vec![0, 4, 1]), "eine Tabelle je Zelle, Index = Kachelnummer");
    assert_eq!(bin(&dir, "t.col.bin"), vec![1, 5, 6]); assert_eq!(bin(&dir, "t.props.bin"), vec![0, 1, 0]);
    assert_eq!(bin(&dir, "lv.map0.bin"), vec![0, 1, 2, 2, 1, 0], "zeilenweise");
    assert!(!dir.join("build/export/lv.map0_hi.bin").exists(), "kein Hochbyte bei 8 Bit");
}

#[test]
fn interleaved_legt_alle_zellen_einer_kachel_hintereinander() {
    let dir = projekt("inter", json!({})); let lay = TilesLayout { interleaved: true, ..Default::default() };
    exportiere(&dir, "graphics/t.vctiles", "t", Some(lay));
    assert_eq!(bin(&dir, "t.t.bin"), vec![0, 0, 0, 0, 1, 2, 3, 4, 4, 3, 2, 1], "Kachel für Kachel, je B×H Byte");
    assert!(!dir.join("build/export/t.c0.bin").exists(), "keine planaren Tabellen"); assert_eq!(bin(&dir, "t.col.bin"), vec![1, 5, 6], "Farbe je Kachel bleibt");
}

#[test]
fn karte_spaltenweise_und_16_bit_index() {
    let dir = projekt("karte", json!({}));
    exportiere(&dir, "graphics/lv.vcscreen", "lv", Some(TilesLayout { map_columns: true, ..Default::default() }));
    assert_eq!(bin(&dir, "lv.map0.bin"), vec![0, 2, 1, 1, 2, 0], "spaltenweise: erst die linke Spalte von oben nach unten");
    let dir = projekt("karte16", json!({})); let r = exportiere(&dir, "graphics/lv.vcscreen", "lv", Some(TilesLayout { index16: true, ..Default::default() }));
    assert_eq!((bin(&dir, "lv.map0.bin"), bin(&dir, "lv.map0_hi.bin")), (vec![0, 1, 2, 2, 1, 0], vec![0; 6]), "niederes und oberes Byte getrennt");
    assert!(r["tilesReport"].as_str().unwrap().contains("16-Bit-Index"), "{r}");
}

#[test]
fn layout_in_der_datei_gilt_ohne_dialog_und_der_dialog_uebersteuert_es() {
    let dir = projekt("datei", json!({ "layout": { "tiles": "planar", "map": "columns", "index": 8 } }));
    exportiere(&dir, "graphics/lv.vcscreen", "lv", None); assert_eq!(bin(&dir, "lv.map0.bin"), vec![0, 2, 1, 1, 2, 0], "Layout aus der Datei");
    exportiere(&dir, "graphics/lv.vcscreen", "lv", Some(TilesLayout::default())); assert_eq!(bin(&dir, "lv.map0.bin"), vec![0, 1, 2, 2, 1, 0], "Dialog übersteuert");
    let l = TilesLayout { interleaved: true, map_columns: true, index16: true }; assert_eq!(TilesLayout::aus_medium(&json!({ "layout": l.json().unwrap() })), l, "Rundreise über die Datei");
    assert!(TilesLayout::default().json().is_none());
}

#[test]
fn acht_bit_index_meldet_zu_viele_kacheln_und_tiles_use_zeile_nennt_die_marken() {
    // 300 Kacheln: mit 8-Bit-Index abgelehnt, mit 16-Bit-Index exportiert (Karte mit Kachel 299 → Hochbyte 1)
    let dir = projekt("zuviel", json!({}));
    let mut ts: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/t.vctiles")).unwrap()).unwrap();
    ts["tiles"] = json!((0..300).map(|i| json!({ "name": format!("k{i}"), "cells": [if i % 2 == 0 { 1 } else { 2 }, 0, 0, 0], "col": 1 })).collect::<Vec<_>>()); ts["propDefs"] = json!([]); write(&dir.join("graphics/t.vctiles"), &ts);
    let mut b: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/lv.vcscreen")).unwrap()).unwrap(); b["screens"][0]["tiles"]["grid"] = json!([[0, 299, 2], [2, 1, 0]]); write(&dir.join("graphics/lv.vcscreen"), &b);
    let e = project::export_mit(&dir, &data(), "graphics/lv.vcscreen", "bin", Some("lv"), None, 0, 0, None).unwrap_err(); assert!(e.contains("16-Bit") || e.contains("256"), "{e}");
    exportiere(&dir, "graphics/lv.vcscreen", "lv", Some(TilesLayout { index16: true, ..Default::default() }));
    assert_eq!((bin(&dir, "lv.map0.bin"), bin(&dir, "lv.map0_hi.bin")), (vec![0, 43, 2, 2, 1, 0], vec![0, 1, 0, 0, 0, 0]));
    // tiles_use-Zeile
    let dir = projekt("use", json!({})); let r = exportiere(&dir, "graphics/lv.vcscreen", "lv", None);
    assert_eq!(r["tilesUse"], "tiles_use t_c0, t_col, t_props, t_tw, t_th, t_count, lv_screens_lo, lv_screens_hi, lv_eins_tiles_w, lv_eins_tiles_h");
    let r = exportiere(&dir, "graphics/lv.vcscreen", "lv", Some(TilesLayout { interleaved: true, ..Default::default() })); assert!(r["tilesUse"].as_str().unwrap().starts_with("tiles_use t_t, t_col,"), "{r}");
}

#[test]
fn mehr_als_255_patches_werden_zu_vollen_zeilen() {
    let dir = projekt("zeilen", json!({}));
    let mut b: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/lv.vcscreen")).unwrap()).unwrap();
    b["screens"][0]["cells"] = json!({ "patches": (0..300).map(|i| json!([i % 6, (i / 6) % 4, 1 + (i % 2)])).collect::<Vec<_>>() }); write(&dir.join("graphics/lv.vcscreen"), &b);
    exportiere(&dir, "graphics/lv.vcscreen", "lv", None);
    let r = bin(&dir, "lv.rows0.bin"); let mut soll = vec![4u8];
    for y in 0..4u8 { soll.push(y); for x in 0..6u8 { soll.push(1 + ((y * 6 + x) % 2)) } soll.extend([1u8; 6]) }
    assert_eq!(r, soll, "jede Zeile als Ganzes: y, Codes, Farben (die letzte Angabe je Zelle gilt)");
    assert!(!dir.join("build/export/lv.cells0.bin").exists());
    let asm = { let _ = project::export(&dir, &data(), "graphics/lv.vcscreen", "asm", Some("lv"), None, 0, 0).unwrap(); std::fs::read_to_string(dir.join("build/export/lv.asm")).unwrap() };
    assert!(asm.contains("lv_rows_lo:") && asm.contains("<lv_rows0"), "Zeigertabelle der Zeilen");
}

#[test]
fn bank_mit_zwei_kachelsaetzen_hat_einen_zielzeichensatz_und_satz_je_screen() {
    let dir = projekt("zweisatz", json!({}));
    // zweiter Zeichensatz b: Glyph 1 hat ein eigenes Muster; zweiter Kachelsatz u: eine Kachel aus Glyph 1 von b
    let frames: Vec<Value> = (0..256).map(|i| json!({ "rows": (0..8).map(|y| if i == 1 && y == 0 { "10101010".to_string() } else { "00000000".to_string() }).collect::<Vec<_>>() })).collect();
    write(&dir.join("graphics/b.vcchar"), &json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": "b", "colors": {}, "frames": frames }));
    write(&dir.join("graphics/u.vctiles"), &json!({ "format": 1, "kind": "tileset", "profile": "c64.tileset", "name": "u", "size": { "w": 2, "h": 2 }, "colorMode": "tile", "charsets": ["graphics/b.vcchar"], "propDefs": [], "tiles": [{ "name": "z", "cells": [1, 1, 1, 1], "col": 2 }] }));
    let mut b: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/lv.vcscreen")).unwrap()).unwrap();
    b["format"] = json!(2); b["tilesets"] = json!(["graphics/t.vctiles", "graphics/u.vctiles"]);
    b["screens"].as_array_mut().unwrap().push(json!({ "name": "zwei", "w": 4, "h": 2, "colors": {}, "tiles": { "set": 1, "origin": [0, 0], "grid": [[0, 0]] } }));
    write(&dir.join("graphics/lv.vcscreen"), &b);
    let r = exportiere(&dir, "graphics/lv.vcscreen", "lv", None);
    assert_eq!(bin(&dir, "lv.map1.bin"), vec![0, 0], "Karte des zweiten Screens: Kacheln des zweiten Satzes"); assert_eq!(bin(&dir, "lv.map0.bin"), vec![0, 1, 2, 2, 1, 0]);
    let (s0, s1, chars) = (bin(&dir, "lv.s0_c0.bin"), bin(&dir, "lv.s1_c0.bin"), bin(&dir, "lv.chars.bin"));
    assert_eq!(s0, vec![0, 1, 4], "Satz 0 behält seine Codes"); assert!(s1[0] > 4, "das Glyph aus b bekommt einen neuen Code: {s1:?}"); assert_eq!(chars[s1[0] as usize * 8], 0b1010_1010, "ein gemeinsamer Zielzeichensatz");
    assert_eq!(bin(&dir, "lv.s1_col.bin"), vec![2]);
    project::export(&dir, &data(), "graphics/lv.vcscreen", "asm", Some("lv"), None, 0, 0).unwrap(); let asm = std::fs::read_to_string(dir.join("build/export/lv.asm")).unwrap();
    assert!(asm.contains("lv_screen_set:\n        .byte 0,1") && asm.contains(".const lv_sets = 2") && asm.contains(".const lv_s1_count = 1"), "{asm}");
    assert!(r["tilesUse"].as_str().unwrap().starts_with("tiles_use lv_s0_c0,"), "{r}");
    // ungültige Satznummer
    b["screens"][1]["tiles"]["set"] = json!(5); write(&dir.join("graphics/lv.vcscreen"), &b);
    assert!(project::export_mit(&dir, &data(), "graphics/lv.vcscreen", "bin", Some("lv"), None, 0, 0, None).unwrap_err().contains("gibt es nicht"));
}
