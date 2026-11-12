// SPDX-License-Identifier: GPL-3.0-or-later
//! Profile mit `extends`, Paletten je System, Systemkompatibilität und Spritemodi aus den Profildaten.
use serde_json::{json, Value};
use std::path::PathBuf;
use vc_media::{load_profiles, palette, profile, project, sprite};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

/// Die aufgelösten Profile müssen den ausgeschriebenen Profilen vor der Einführung von `extends` entsprechen
/// (Stand `tests/profiles.resolved.json`); neu sind nur `perFrameMode` und `modes` der C64-Sprites.
#[test]
fn resolved_profiles_match_the_golden_file() {
    let golden: Value = serde_json::from_str(&std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/profiles.resolved.json")).unwrap()).unwrap();
    let want = golden["profiles"].as_array().unwrap();
    let got = load_profiles(&data()).unwrap();
    assert_eq!(got.len(), want.len());
    for (g, w) in got.iter().zip(want) {
        let mut raw = g.raw.clone();
        if let Some(o) = raw.as_object_mut() { o.remove("perFrameMode"); o.remove("modes"); }
        assert_eq!(&raw, w, "Profil {} weicht ab", g.id);
        assert!(g.raw.get("extends").is_none(), "extends muss aufgelöst sein ({})", g.id);
    }
}

#[test]
fn extends_overrides_only_the_changed_fields() {
    let all = vec![json!({"id": "a", "kind": "charset", "width": 8, "bpp": 1, "name": "A"}), json!({"id": "b", "extends": "a", "name": "B", "width": 4})];
    let r = vc_media::resolve_profile_json(&all, &all[1], 0).unwrap();
    assert_eq!(r, json!({"id": "b", "kind": "charset", "width": 4, "bpp": 1, "name": "B"}));
}

#[test]
fn extends_rejects_unknown_parent_and_cycles() {
    let unknown = vec![json!({"id": "x", "extends": "gibtsnicht"})];
    assert!(vc_media::resolve_profile_json(&unknown, &unknown[0], 0).unwrap_err().contains("unbekanntes Profil"));
    let cycle = vec![json!({"id": "a", "extends": "b"}), json!({"id": "b", "extends": "a"})];
    assert!(vc_media::resolve_profile_json(&cycle, &cycle[0], 0).is_err());
}

#[test]
fn palettes_come_from_one_place() {
    let d = data();
    let c64 = palette::palette_for(&d, "c64", "pal").unwrap();
    assert_eq!(c64.colors.len(), 16); assert_eq!(c64.rgb(1), Some([0xff, 0xff, 0xff]));
    assert_eq!(c64.rgb(17), c64.rgb(1), "Farbwert wird auf 4 Bit maskiert");
    // C128 und Plus/4 benutzen (bis es eine TED-Palette gibt) die C64-Farben, unbekannte Systeme ebenfalls
    assert_eq!(palette::palette_for(&d, "c128", "pal").unwrap(), c64);
    assert_eq!(palette::palette_for(&d, "plus4", "pal").unwrap(), c64);
    assert_eq!(palette::palette_for(&d, "gibtsnicht", "pal").unwrap(), c64);
    // Atari: GTIA-Wert = Zeile · 2, PAL und NTSC unterscheiden sich
    let pal = palette::palette_for(&d, "atari8", "pal").unwrap(); let ntsc = palette::palette_for(&d, "atari8", "ntsc").unwrap();
    assert_eq!(pal.colors.len(), 128); assert_eq!(pal.step, 2);
    assert_eq!(pal.rgb(2), Some(pal.colors[1])); assert_eq!(pal.entries()[1].0, 2); assert_eq!(pal.entries()[127].0, 254);
    assert_ne!(pal.colors, ntsc.colors);
    // MEGA65: berechnete Standardpalette mit 256 Einträgen
    let m65 = palette::palette_for(&d, "mega65", "pal").unwrap();
    assert_eq!(m65.colors.len(), 256); assert_eq!(m65.rgb(255), m65.colors.last().copied());
    assert_eq!(vc_media::import::palette(&d, "atari8", "ntsc").unwrap(), ntsc.entries());
    assert!(palette::known_targets(&d).contains(&"vic20".to_string()));
}

#[test]
fn compatible_systems_extend_the_project_systems() {
    let d = data();
    assert_eq!(project::compatible_systems(&d, "c128"), vec!["c64".to_string()]);
    assert!(project::compatible_systems(&d, "atari8").is_empty());
    let dir = std::env::temp_dir().join(format!("vc_compat_{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("p.vcprj"), r#"{"targets":[{"system":"c128"}]}"#).unwrap();
    let sys = project::project_systems(&dir, &d);
    assert!(sys.contains(&"c128".to_string()) && sys.contains(&"c64".to_string()), "{sys:?}");
    std::fs::write(dir.join("p.vcprj"), r#"{"targets":[{"system":"atari8"}]}"#).unwrap();
    assert_eq!(project::project_systems(&dir, &d), vec!["atari8".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sprite_modes_come_from_the_profile() {
    let ps = load_profiles(&data()).unwrap();
    let hires = profile(&ps, "c64.sprite.hires").unwrap(); let mc = profile(&ps, "c64.sprite.multicolor").unwrap();
    assert!(sprite::has_frame_modes(hires) && sprite::has_frame_modes(mc), "multicolor erbt perFrameMode und modes");
    let (h, m) = (json!({ "rows": [] }), json!({ "mode": "multicolor", "rows": [] }));
    assert_eq!(sprite::frame_geometry(hires, &h), (24, 1)); assert_eq!(sprite::frame_geometry(hires, &m), (12, 2));
    assert_eq!(sprite::frame_geometry(mc, &h), (12, 2), "ohne mode gilt der Modus des Profils"); assert_eq!(sprite::frame_geometry(mc, &json!({ "rows": [], "mode": "hires" })), (24, 1));
    // Profile ohne Modus je Frame behalten ihre Geometrie
    let pmg = profile(&ps, "atari.pmg.single").unwrap();
    assert!(!sprite::has_frame_modes(pmg)); assert_eq!(sprite::frame_geometry(pmg, &h), (pmg.width, pmg.bpp));
}
