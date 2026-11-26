// SPDX-License-Identifier: GPL-3.0-or-later
//! Fenstergröße und -lage merken (E1.4): beim Beenden gespeichert in `window.json` im Konfigurationsordner, beim Start wiederhergestellt.
//! Liegt das gemerkte Fenster auf keinem Bildschirm mehr (anderer Monitor abgesteckt), kommt es mittig auf den Hauptbildschirm; auf einem kleinen
//! Bildschirm startet das Fenster maximiert. Die Entscheidung (`plan`) ist reine Rechnung und getestet; das Anwenden steht in `anwenden`.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const STANDARD: (u32, u32) = (1440, 900);
pub const MINDEST: (u32, u32) = (900, 600);
/// Kleiner als das (logisch): das Fenster startet maximiert
pub const KLEIN: (u32, u32) = (1280, 800);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zustand { pub x: i32, pub y: i32, pub w: u32, pub h: u32, #[serde(default)] pub maximiert: bool, /** Skalierung des Bildschirms beim Speichern (0 = unbekannt, alte Datei): x, y, w, h sind physisch auf diesem Bildschirm */ #[serde(default)] pub skala: f64 }

/// Ein Bildschirm: linke obere Ecke und Größe in physischen Punkten, dazu der Skalierungsfaktor
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Schirm { pub x: i32, pub y: i32, pub w: u32, pub h: u32, pub skala: f64 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan { pub pos: Option<(i32, i32)>, pub groesse: (u32, u32), pub maximiert: bool }

pub fn datei() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("window.json")) }
pub fn lesen(p: &Path) -> Option<Zustand> { serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok() }
pub fn schreiben(p: &Path, z: &Zustand) -> bool {
    if let Some(d) = p.parent() { let _ = std::fs::create_dir_all(d); }
    serde_json::to_string_pretty(z).map(|t| vc_core::atomar::schreiben(p, t + "\n").is_ok()).unwrap_or(false)
}

fn schnitt(z: &Zustand, s: &Schirm) -> (i64, i64) {
    let (x0, y0, x1, y1) = (z.x.max(s.x) as i64, z.y.max(s.y) as i64, (z.x as i64 + z.w as i64).min(s.x as i64 + s.w as i64), (z.y as i64 + z.h as i64).min(s.y as i64 + s.h as i64));
    ((x1 - x0).max(0), (y1 - y0).max(0))
}

/// Was beim Start geschieht. `schirme[0]` ist der Hauptbildschirm.
pub fn plan(gemerkt: Option<Zustand>, schirme: &[Schirm]) -> Plan {
    let Some(haupt) = schirme.first().copied() else { return Plan { pos: None, groesse: STANDARD, maximiert: false } };
    let klein = |s: &Schirm| (s.w as f64 / s.skala) < KLEIN.0 as f64 || (s.h as f64 / s.skala) < KLEIN.1 as f64;
    let Some(z) = gemerkt else { return Plan { pos: None, groesse: STANDARD, maximiert: klein(&haupt) } };
    let groesse = (z.w.max((MINDEST.0 as f64 * haupt.skala) as u32), z.h.max((MINDEST.1 as f64 * haupt.skala) as u32));
    // Mindestens ein Stück des Titelbereichs (200 × 60 Punkte) muss auf einem Bildschirm liegen, sonst käme das Fenster nicht mehr zu greifen
    if schirme.iter().any(|s| { let (b, h) = schnitt(&Zustand { w: groesse.0, h: groesse.1, ..z }, s); b >= 200 && h >= 60 }) { Plan { pos: Some((z.x, z.y)), groesse, maximiert: z.maximiert } }
    else { Plan { pos: None, groesse: (groesse.0.min(haupt.w), groesse.1.min(haupt.h)), maximiert: z.maximiert || klein(&haupt) } }
}

/// Alles in logischen Punkten (macOS und gemischte Skalierung): Bildschirme mit `position / skala` und `größe / skala`, das gemerkte Fenster mit der Skalierung, bei der es gespeichert wurde.
/// Unter macOS sind Bildschirme und Fenster logisch angeordnet; physische Werte verschiedener Bildschirme (Skala 2 und 1) wären nicht vergleichbar. Das Ergebnis (`Plan`) ist dann ebenfalls logisch.
pub fn schirme_logisch(schirme: &[Schirm]) -> Vec<Schirm> {
    schirme.iter().map(|s| { let k = if s.skala > 0.0 { s.skala } else { 1.0 }; Schirm { x: (s.x as f64 / k).round() as i32, y: (s.y as f64 / k).round() as i32, w: (s.w as f64 / k).round() as u32, h: (s.h as f64 / k).round() as u32, skala: 1.0 } }).collect()
}
pub fn zustand_logisch(z: &Zustand, haupt_skala: f64) -> Zustand {
    let k = if z.skala > 0.0 { z.skala } else if haupt_skala > 0.0 { haupt_skala } else { 1.0 };
    Zustand { x: (z.x as f64 / k).round() as i32, y: (z.y as f64 / k).round() as i32, w: (z.w as f64 / k).round() as u32, h: (z.h as f64 / k).round() as u32, maximiert: z.maximiert, skala: 1.0 }
}
pub fn plan_logisch(gemerkt: Option<Zustand>, schirme: &[Schirm]) -> Plan {
    let haupt = schirme.first().map_or(1.0, |s| s.skala);
    plan(gemerkt.map(|z| zustand_logisch(&z, haupt)), &schirme_logisch(schirme))
}

#[cfg(test)]
mod tests {
    use super::*;
    const A: Schirm = Schirm { x: 0, y: 0, w: 3840, h: 2160, skala: 2.0 };
    const B: Schirm = Schirm { x: 3840, y: 0, w: 1920, h: 1080, skala: 1.0 };
    #[test] fn ohne_gemerktes_fenster_standardgroesse_und_kleiner_schirm_maximiert() {
        assert_eq!(plan(None, &[A]), Plan { pos: None, groesse: STANDARD, maximiert: false });
        assert!(plan(None, &[Schirm { x: 0, y: 0, w: 1366, h: 768, skala: 1.0 }]).maximiert, "1366×768 gilt als klein");
        assert!(plan(None, &[Schirm { x: 0, y: 0, w: 2560, h: 1440, skala: 2.0 }]).maximiert, "2560×1440 bei Skalierung 2 sind nur 1280×720 Punkte: klein");
    }
    #[test] fn gemerktes_fenster_auf_vorhandenem_schirm_wird_uebernommen() {
        let z = Zustand { x: 4000, y: 100, w: 1500, h: 900, maximiert: false, skala: 0.0 };
        assert_eq!(plan(Some(z), &[A, B]), Plan { pos: Some((4000, 100)), groesse: (1800, 1200), maximiert: false }, "unter der Mindestgröße (900×600 Punkte, hier bei Skalierung 2) wird sie angehoben");
        let z = Zustand { x: 4000, y: 100, w: 1900, h: 1250, maximiert: false, skala: 0.0 }; assert_eq!(plan(Some(z), &[A, B]).groesse, (1900, 1250));
    }
    #[test] fn abgestecktes_zweites_geraet_bringt_das_fenster_auf_den_hauptschirm() {
        let z = Zustand { x: 4000, y: 100, w: 1900, h: 1250, maximiert: false, skala: 0.0 };
        let p = plan(Some(z), &[A]); assert_eq!(p.pos, None, "mittig auf dem Hauptschirm"); assert_eq!(p.groesse, (1900, 1250));
    }
    #[test] fn zu_kleine_oder_zu_grosse_groessen_werden_begrenzt() {
        let p = plan(Some(Zustand { x: 0, y: 0, w: 100, h: 100, maximiert: false, skala: 0.0 }), &[Schirm { x: 0, y: 0, w: 1920, h: 1080, skala: 1.0 }]); assert_eq!(p.groesse, MINDEST);
        let p = plan(Some(Zustand { x: 50000, y: 0, w: 5000, h: 5000, maximiert: true, skala: 0.0 }), &[Schirm { x: 0, y: 0, w: 1920, h: 1080, skala: 1.0 }]); assert_eq!((p.pos, p.groesse, p.maximiert), (None, (1920, 1080), true));
    }
    #[test] fn zustand_wird_geschrieben_und_gelesen_und_kaputtes_wird_ignoriert() {
        let d = std::env::temp_dir().join(format!("vc_fenster_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); let f = d.join("window.json");
        let z = Zustand { x: -10, y: 20, w: 1300, h: 800, maximiert: true, skala: 0.0 };
        assert!(schreiben(&f, &z)); assert_eq!(lesen(&f), Some(z));
        std::fs::write(&f, "kaputt").unwrap(); assert_eq!(lesen(&f), None);
        let _ = std::fs::remove_dir_all(&d);
    }
    // Gemischte Skalierung wie am Mac: Hauptschirm eingebaut (3024×1964, Skala 2 = 1512×982 Punkte), zweiter Schirm 3440×1440 (Skala 1) rechts, links oder darüber.
    // Die physische Position eines Schirms ist (wie bei tao unter macOS) die logische Position mal seine Skala.
    fn haupt() -> Schirm { Schirm { x: 0, y: 0, w: 3024, h: 1964, skala: 2.0 } }
    #[test] fn zweiter_schirm_mit_anderer_skala_rechts_wird_logisch_getroffen() {
        let b = Schirm { x: 1512, y: 0, w: 3440, h: 1440, skala: 1.0 };
        let z = Zustand { x: 1700, y: 100, w: 1800, h: 1000, maximiert: false, skala: 1.0 };   // gespeichert auf dem zweiten Schirm
        let p = plan_logisch(Some(z), &[haupt(), b]);
        assert_eq!(p, Plan { pos: Some((1700, 100)), groesse: (1800, 1000), maximiert: false }, "bleibt auf dem zweiten Schirm, Größe nicht halbiert");
    }
    #[test] fn zweiter_schirm_links_und_oben_mit_negativen_oder_grossen_werten() {
        let links = Schirm { x: -3440, y: 0, w: 3440, h: 1440, skala: 1.0 };
        let z = Zustand { x: -3000, y: 50, w: 2000, h: 1100, maximiert: false, skala: 1.0 };
        assert_eq!(plan_logisch(Some(z), &[haupt(), links]).pos, Some((-3000, 50)));
        let oben = Schirm { x: 100, y: -1440, w: 3440, h: 1440, skala: 1.0 };
        let z = Zustand { x: 300, y: -1300, w: 1600, h: 1000, maximiert: false, skala: 1.0 };
        assert_eq!(plan_logisch(Some(z), &[haupt(), oben]).pos, Some((300, -1300)));
    }
    #[test] fn gespeichert_auf_dem_hauptschirm_mit_skala_2_wird_in_punkten_gelesen() {
        let b = Schirm { x: 1512, y: 0, w: 3440, h: 1440, skala: 1.0 };
        let z = Zustand { x: 400, y: 200, w: 2000, h: 1400, maximiert: false, skala: 2.0 };   // physisch auf dem Retina-Schirm = 200,100 / 1000×700 Punkte
        let p = plan_logisch(Some(z), &[haupt(), b]); assert_eq!(p.pos, Some((200, 100))); assert_eq!(p.groesse, (1000, 700));
    }
    #[test] fn zweiter_schirm_abgesteckt_bringt_das_fenster_auf_den_hauptschirm_in_punkten() {
        let z = Zustand { x: 1700, y: 100, w: 1800, h: 1000, maximiert: false, skala: 1.0 };
        let p = plan_logisch(Some(z), &[haupt()]); assert_eq!(p.pos, None); assert_eq!(p.groesse, (1512, 982), "auf die Größe des Hauptschirms (in Punkten) begrenzt");
    }
    #[test] fn alte_datei_ohne_skala_gilt_als_hauptschirm_skala() {
        let z: Zustand = serde_json::from_str(r#"{"x":400,"y":200,"w":2000,"h":1400}"#).unwrap(); assert_eq!(z.skala, 0.0);
        let p = plan_logisch(Some(z), &[haupt()]); assert_eq!(p.pos, Some((200, 100)));
    }
}
