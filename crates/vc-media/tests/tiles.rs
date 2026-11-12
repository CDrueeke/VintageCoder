// SPDX-License-Identifier: GPL-3.0-or-later
//! Kachelsatz und Screen-Bank: Umsetzung der Jumprun-Karten, Export, Zeichensatz-Zusammenführung.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_media::{load_profiles, parse_media, profile, project, tiles};

fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
fn data() -> PathBuf { repo().join("data") }
fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-tiles-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("graphics")).unwrap(); d }
fn read(p: &Path) -> Value { serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap() }
fn write(p: &Path, v: &Value) { std::fs::write(p, project::format_media(v)).unwrap(); }

/// Jumprun C64: vier alte Karten → ein Kachelsatz + eine Bank mit vier Screens.
/// Das Beispiel selbst ist inzwischen migriert (hg.vctiles + levels.vcscreen); die alten Karten (Art `tilemap`, je Bild eine Datei) werden daraus
/// zurückgerechnet, damit der Umsetzer weiter an echten Daten geprüft wird.
fn jumprun(dir: &Path) -> (Vec<Value>, Value, Value) {
    let src = repo().join("examples/c64/jumprun/graphics");
    std::fs::copy(src.join("chars.vcchar"), dir.join("graphics/chars.vcchar")).unwrap();
    let (cur_set, cur_bank) = (read(&src.join("hg.vctiles")), read(&src.join("levels.vcscreen")));
    let old_tiles: Vec<Value> = cur_set["tiles"].as_array().unwrap().iter().map(|t| json!({
        "name": t["name"], "chars": t["cells"], "color": t["col"], "solid": t["props"].as_array().map_or(false, |p| p.iter().any(|x| x == "solid")) })).collect();
    let olds: Vec<Value> = cur_bank["screens"].as_array().unwrap().iter().map(|sc| json!({
        "format": 1, "kind": "tilemap", "profile": "c64.tilemap.2x2", "name": sc["name"], "colors": sc["colors"], "charset": "graphics/chars.vcchar",
        "tiles": old_tiles, "map": sc["tiles"]["grid"] })).collect();
    assert_eq!(olds.len(), 4);
    let mut set = Value::Null; let mut screens = vec![];
    for o in &olds { let (s, sc) = tiles::migrate_tilemap(o, 2, 2, true, "hg").unwrap(); if set.is_null() { set = s; } else { assert_eq!(set["tiles"], s["tiles"], "alle vier Bilder haben denselben Satz"); } screens.push(sc); }
    let bank = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "levels", "tilesets": ["graphics/hg.vctiles"], "screens": screens });
    write(&dir.join("graphics/hg.vctiles"), &set); write(&dir.join("graphics/levels.vcscreen"), &bank);
    (olds, set, bank)
}

#[test]
fn migrated_set_and_bank_export_matches_the_old_export() {
    let dir = tmp("jr"); let (olds, set, _bank) = jumprun(&dir); let d = data();
    assert_eq!(set["size"], json!({ "w": 2, "h": 2 })); assert_eq!(set["tiles"].as_array().unwrap().len(), 21);
    project::export(&dir, &d, "graphics/hg.vctiles", "bin", Some("hg"), None, 0, 0).unwrap();
    project::export(&dir, &d, "graphics/levels.vcscreen", "bin", Some("levels"), None, 0, 0).unwrap();
    let rd = |n: &str| std::fs::read(dir.join("build/export").join(n)).unwrap();
    // Glyphen: jede Zelle zeigt nach der Umnummerierung dasselbe Muster wie vorher
    let profiles = load_profiles(&d).unwrap();
    let cs_media = parse_media(&std::fs::read_to_string(dir.join("graphics/chars.vcchar")).unwrap()).unwrap();
    let orig = vc_media::encode(&cs_media, profile(&profiles, &cs_media.profile).unwrap()).unwrap();
    let merged = rd("hg.chars.bin"); assert_eq!(merged.len(), orig.len());
    let tiles_arr = olds[0]["tiles"].as_array().unwrap();
    for k in 0..4 {
        let table = rd(&format!("hg.c{k}.bin")); assert_eq!(table.len(), 21);
        for (i, t) in tiles_arr.iter().enumerate() {
            let old_code = t["chars"][k].as_u64().unwrap() as usize; let new_code = table[i] as usize;
            assert_eq!(merged[new_code * 8..new_code * 8 + 8], orig[old_code * 8..old_code * 8 + 8], "Kachel {i} Zelle {k}");
        }
    }
    assert_eq!(rd("hg.col.bin"), tiles_arr.iter().map(|t| t["color"].as_u64().unwrap() as u8).collect::<Vec<_>>());
    assert_eq!(rd("hg.props.bin"), tiles_arr.iter().map(|t| t["solid"].as_bool().unwrap() as u8).collect::<Vec<_>>());
    // Karten: byte-gleich zu den alten
    for (i, o) in olds.iter().enumerate() {
        let want: Vec<u8> = o["map"].as_array().unwrap().iter().flat_map(|r| r.as_array().unwrap().iter().map(|c| c.as_u64().unwrap() as u8)).collect();
        assert_eq!(rd(&format!("levels.map{i}.bin")), want, "Karte {i}");
    }
}

#[test]
fn asm_export_has_tables_constants_and_screen_table() {
    let dir = tmp("asm"); jumprun(&dir); let d = data();
    project::export(&dir, &d, "graphics/hg.vctiles", "asm", Some("hg"), None, 0, 0).unwrap();
    project::export(&dir, &d, "graphics/levels.vcscreen", "asm", Some("levels"), None, 0, 0).unwrap();
    let hg = std::fs::read_to_string(dir.join("build/export/hg.asm")).unwrap();
    for w in ["hg_c0:", "hg_c3:", "hg_col:", "hg_props:", "hg_chars:", ".const hg_tw = 2", ".const hg_count = 21", ".const hg_p_solid = 1"] { assert!(hg.contains(w), "hg.asm: {w}\n{hg}"); }
    let lv = std::fs::read_to_string(dir.join("build/export/levels.asm")).unwrap();
    for w in ["levels_map0:", "levels_map3:", "levels_screens_lo:", ".byte <levels_map0,<levels_map1,<levels_map2,<levels_map3", ".byte >levels_map0", ".const levels_lv2_id = 1", ".const levels_count = 4", ".const levels_lv1_tiles_w = 20", ".const levels_lv1_tiles_h = 12", ".const levels_lv1_bg = 3"] { assert!(lv.contains(w), "levels.asm: {w}"); }
    project::export(&dir, &d, "graphics/levels.vcscreen", "sbas", Some("levels"), None, 0, 0).unwrap();
    assert!(std::fs::read_to_string(dir.join("build/export/levels.data.sbas")).unwrap().contains("const levels_count = 4"));
}

fn mini_charset(dir: &Path, name: &str, glyphs: &[&str]) { mini_charset_at(dir, name, glyphs, 0) }
fn mini_charset_at(dir: &Path, name: &str, glyphs: &[&str], row: usize) {
    // c64.char.hires: 256 Zeichen, 8×8; glyphs[i] ist die Zeichennummer i (erste Zeile mit '1' an Stelle i%8, sonst leer)
    let frames: Vec<Value> = (0..256).map(|i| { let on = glyphs.get(i).copied().unwrap_or(""); json!({ "rows": (0..8).map(|y| if on.is_empty() { "00000000".to_string() } else if y == row { on.to_string() } else { "00000000".to_string() }).collect::<Vec<_>>() }) }).collect();
    write(&dir.join(format!("graphics/{name}.vcchar")), &json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": name, "colors": {}, "frames": frames }));
}
fn set_with(cells: Value, extra: Value) -> Value {
    let mut v = json!({ "format": 1, "kind": "tileset", "profile": "c64.tileset", "name": "t", "size": { "w": 2, "h": 1 }, "colorMode": "tile",
        "charsets": ["graphics/a.vcchar", "graphics/b.vcchar"], "propDefs": [], "tiles": [{ "name": "x", "cells": cells, "col": 5 }] });
    for (k, x) in extra.as_object().unwrap() { v[k] = x.clone(); } v
}

#[test]
fn glyphs_from_two_charsets_are_merged_and_duplicates_collapse() {
    let dir = tmp("merge"); let d = data();
    mini_charset(&dir, "a", &["", "11110000", "00001111"]);
    mini_charset(&dir, "b", &["", "11110000", "10101010"]);
    // A:1 (11110000) = B:1 (gleiches Muster); B:2 neu; A:0/B:0 leer → Code 0
    write(&dir.join("graphics/t.vctiles"), &set_with(json!([[1, 0], [1, 1]]), json!({ "tiles": [{ "name": "x", "cells": [[1, 0], [1, 1]], "col": 5 }, { "name": "y", "cells": [[2, 1], [0, 1]], "col": 6 }, { "name": "z", "cells": [[0, 0], [2, 0]], "col": 7 }] })));
    project::export(&dir, &d, "graphics/t.vctiles", "bin", Some("t"), None, 0, 0).unwrap();
    let rd = |n: &str| std::fs::read(dir.join("build/export").join(n)).unwrap();
    let (c0, c1, chars) = (rd("t.c0.bin"), rd("t.c1.bin"), rd("t.chars.bin"));
    assert_eq!(c0[0], c1[0], "gleiches Muster aus zwei Zeichensätzen = ein Zielzeichen");
    assert_eq!(c0[2], 0, "leeres Glyph ist Code 0"); assert_eq!(c1[1], 0);
    assert_ne!(c1[1], c0[0]); assert_eq!(chars[c0[0] as usize * 8], 0b1111_0000); assert_eq!(chars[c0[1] as usize * 8], 0b1010_1010);
    assert_eq!(chars[c1[2] as usize * 8], 0b0000_1111, "A:2 ist ein drittes Glyph");
}

#[test]
fn overflow_and_validation_errors_are_clear() {
    let dir = tmp("err"); let d = data();
    let many: Vec<String> = (0..256).map(|i| format!("{:08b}", i)).collect(); let refs: Vec<&str> = many.iter().map(|s| s.as_str()).collect();
    mini_charset(&dir, "a", &refs); mini_charset(&dir, "b", &["11111111"]);
    // 255 Kacheln mit je einem eigenen Glyph + ein Blank → mehr als 256 verschiedene gibt es nur mit zwei Sätzen: A (256 verschiedene) + B:0 (11111111 = A:255)
    let cells: Vec<Value> = (1..=255).map(|i| json!({ "name": format!("t{i}"), "cells": [[i, 0], [i, 0]], "col": 1 })).collect();
    let mut v = set_with(json!([[1, 0], [1, 0]]), json!({ "tiles": cells })); v["charsets"] = json!(["graphics/a.vcchar"]);
    write(&dir.join("graphics/ok.vctiles"), &v); project::export(&dir, &d, "graphics/ok.vctiles", "bin", None, None, 0, 0).expect("255 Glyphen passen");
    // 257 Kacheln: abgelehnt
    let tl: Vec<Value> = (0..257).map(|i| json!({ "name": format!("t{i}"), "cells": [[1, 0], [1, 0]], "col": 1 })).collect();
    write(&dir.join("graphics/big.vctiles"), &set_with(json!([[1, 0], [1, 0]]), json!({ "tiles": tl })));
    assert!(project::export(&dir, &d, "graphics/big.vctiles", "bin", None, None, 0, 0).unwrap_err().contains("höchstens 256"));
    // falsche Zellenzahl, unbekannter Zeichensatz, unbekannte Eigenschaft
    assert!(tiles::parse_tileset(&set_with(json!([[1, 0]]), json!({}))).unwrap_err().contains("1 statt 2 Zellen"));
    assert!(tiles::parse_tileset(&set_with(json!([[1, 5], [1, 0]]), json!({}))).unwrap_err().contains("Zeichensatz 5"));
    assert!(tiles::parse_tileset(&set_with(json!([1, 1]), json!({ "tiles": [{ "name": "x", "cells": [1, 1], "props": ["fliegt"] }] }))).unwrap_err().contains("fliegt"));
    assert!(tiles::parse_tileset(&json!({ "kind": "tileset", "size": { "w": 0, "h": 2 }, "tiles": [] })).unwrap_err().contains("1..16"));
    // Überlauf der Glyphen: A hat 256 verschiedene, B bringt ein 257. Muster
    let mut b = vec!["11111111"; 1]; b.push("00000001"); let _ = b;
    mini_charset_at(&dir, "c", &["10000001", "01000001"], 3);
    let mut v = set_with(json!([[1, 0], [1, 0]]), json!({ "tiles": (1..=255).map(|i| json!({ "name": "t", "cells": [[i, 0], [i % 2, 1]], "col": 1 })).collect::<Vec<_>>() }));
    v["charsets"] = json!(["graphics/a.vcchar", "graphics/c.vcchar"]);
    write(&dir.join("graphics/ov.vctiles"), &v);
    let e = project::export(&dir, &d, "graphics/ov.vctiles", "bin", None, None, 0, 0).unwrap_err(); assert!(e.contains("Überlauf"), "{e}");
}

#[test]
fn bank_validation_and_cell_patches() {
    let dir = tmp("bank"); let d = data();
    mini_charset(&dir, "a", &["", "11110000"]);
    let mut set = set_with(json!([[1, 0], [1, 0]]), json!({ "size": { "w": 2, "h": 1 } })); set["charsets"] = json!(["graphics/a.vcchar"]);
    write(&dir.join("graphics/t.vctiles"), &set);
    let bank = |grid: Value, w: u64| json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "b", "tilesets": ["graphics/t.vctiles"],
        "screens": [{ "name": "s", "w": w, "h": 2, "colors": { "bg": 6 }, "tiles": { "set": 0, "origin": [0, 0], "grid": grid }, "cells": { "patches": [[0, 1, { "c": 1, "col": 7 }]] } }] });
    write(&dir.join("graphics/b.vcscreen"), &bank(json!([[0, 0, 0]]), 6));
    project::export(&dir, &d, "graphics/b.vcscreen", "bin", None, None, 0, 0).unwrap();
    let cells = std::fs::read(dir.join("build/export/b.cells0.bin")).unwrap(); assert_eq!(cells.len(), 5); assert_eq!(cells[0], 1); assert_eq!(&cells[1..3], &[0, 1]); assert_eq!(cells[4], 7);
    write(&dir.join("graphics/b.vcscreen"), &bank(json!([[0, 0, 0]]), 5));
    assert!(project::export(&dir, &d, "graphics/b.vcscreen", "bin", None, None, 0, 0).unwrap_err().contains("passt nicht"));
    write(&dir.join("graphics/b.vcscreen"), &bank(json!([[0, 7, 0]]), 6));
    assert!(project::export(&dir, &d, "graphics/b.vcscreen", "bin", None, None, 0, 0).unwrap_err().contains("Kachel 7"));
}

#[test]
fn atari_sel_sets_bit_7_and_needs_128_glyphs() {
    let dir = tmp("atari"); let d = data();
    // atari.char.antic4: 128 Zeichen, 4×8, 2 Bit je Pixel (Ziffern 0..3)
    let frames: Vec<Value> = (0..128).map(|i| json!({ "rows": (0..8).map(|_| if i == 1 { "1111" } else { "0000" }).collect::<Vec<_>>() })).collect();
    write(&dir.join("graphics/a.vcchar"), &json!({ "format": 1, "kind": "charset", "profile": "atari.char.antic4", "name": "a", "colors": {}, "frames": frames }));
    let set = json!({ "format": 1, "kind": "tileset", "profile": "atari.tileset", "name": "t", "size": { "w": 2, "h": 1 }, "colorMode": "none", "charsets": ["graphics/a.vcchar"],
        "tiles": [{ "name": "x", "cells": [[1], { "c": 1, "sel": true }] }] });
    write(&dir.join("graphics/t.vctiles"), &set);
    project::export(&dir, &d, "graphics/t.vctiles", "bin", None, None, 0, 0).unwrap();
    let c0 = std::fs::read(dir.join("build/export/t.c0.bin")).unwrap(); let c1 = std::fs::read(dir.join("build/export/t.c1.bin")).unwrap();
    assert_eq!(c1[0] & 0x7f, c0[0] & 0x7f); assert_eq!(c1[0] & 0x80, 0x80); assert_eq!(c0[0] & 0x80, 0); assert!(!dir.join("build/export/t.col.bin").exists(), "Atari ohne Farbbyte");
    assert_eq!(std::fs::read(dir.join("build/export/t.chars.bin")).unwrap().len(), 128 * 8);
}

#[test]
fn tileset_json_schreibt_zurueck_und_bleibt_lesbar() {
    let mut v = json!({ "format": 1, "kind": "tileset", "profile": "c64.tileset", "name": "t", "size": { "w": 2, "h": 1 }, "colorMode": "cell", "charsets": ["a.vcchar", "b.vcchar"],
        "propDefs": [{ "name": "solid", "bit": 0 }, { "name": "leiter", "bit": 3 }], "pins": [{ "cs": 0, "from": 32, "to": 40 }],
        "tiles": [{ "name": "x", "cells": [1, { "c": 2, "s": 1, "col": 5, "sel": true }], "col": 3, "props": ["leiter"] }, { "name": "y", "cells": [[4, 1], 0], "col": 1 }] });
    let ts = tiles::parse_tileset(&v).unwrap();
    let out = tiles::tileset_json(&ts, &v);
    assert_eq!(out, v, "unverändert gelesen und geschrieben ergibt dieselbe Datei");
    let mut ts2 = ts.clone(); ts2.tiles[1].props = 1; ts2.color_mode = tiles::ColorMode::None;
    let out2 = tiles::tileset_json(&ts2, &v);
    assert_eq!((out2["colorMode"].clone(), out2["tiles"][1]["props"].clone(), out2["pins"].clone()), (json!("none"), json!(["solid"]), v["pins"].clone()), "pins bleiben");
    v["tiles"][1]["props"] = json!(["solid"]); v["colorMode"] = json!("none");
    assert_eq!(out2, v);
}
