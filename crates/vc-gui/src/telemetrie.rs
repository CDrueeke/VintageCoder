// SPDX-License-Identifier: GPL-3.0-or-later
//! Takt und Raster des Zielsystems als kleine Anzeige in der Statusleiste (Einstellung, Standard aus). Rein lokal: es wird nichts erhoben und nichts gesendet,
//! die Werte stehen in `data/targets/*.target.json`. Angezeigt wird nur, was das Zielsystem sicher hergibt: Takt und Norm immer, Zeilen je Bild und Zyklen je
//! Zeile nur bei Systemen, deren Datei beides als CPU-Zyklen führt (nicht beim MEGA65: dort zählt die Datei Bildpunkte, keine CPU-Zyklen).
use std::path::Path;

/// Kurzname der CPU für die Anzeige
fn cpu_name(id: &str) -> String { match id { "nmos6502" => "6502".into(), "sally" => "6502 (Sally)".into(), "45gs02" => "45GS02".into(), x => x.to_uppercase() } }

/// „6510 · 0,985 MHz · PAL · 312 Zeilen · 63 Zyklen/Zeile“ für das System (`c64`, `vic20` …) und die Norm (`pal`/`ntsc`); `None`, wenn die Zieldatei fehlt oder keinen Takt nennt
pub fn takt_text(data: &Path, system: &str, norm: &str) -> Option<String> {
    let text = std::fs::read_to_string(data.join("targets").join(format!("{system}.target.json"))).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let norm = if norm.eq_ignore_ascii_case("ntsc") { "ntsc" } else { "pal" };
    let var = &v["variants"][norm]; let hz = var["clockHz"].as_f64()?;
    let mut t = vc_i18n::t!("telemetrie-p1-p2-mhz-p3", p1 = cpu_name(v["target"]["cpu"].as_str().unwrap_or("")), p2 = vc_i18n::num(hz / 1_000_000.0, 3), p3 = norm.to_uppercase());
    // Zeilen und Zyklen nur dort, wo beides CPU-Zyklen sind (die Zyklen je Zeile müssen zu Takt und Zeilenfrequenz passen: Takt ≈ Zeilen × Zyklen × Bildrate)
    if let (Some(z), Some(c)) = (var["linesPerFrame"].as_u64(), var["cyclesPerLine"].as_u64()) {
        let bild = hz / (z * c) as f64; if (40.0..75.0).contains(&bild) { t.push_str(&format!(" {}", vc_i18n::t!("telemetrie-z-zeilen-c-zyklen-zeile", z = &z, c = &c))) }
    }
    Some(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
    #[test] fn c64_pal_und_ntsc_mit_zeilen_und_zyklen() {
        assert_eq!(takt_text(&data(), "c64", "pal").as_deref(), Some("6510 · 0,985 MHz · PAL · 312 Zeilen · 63 Zyklen/Zeile"));
        assert_eq!(takt_text(&data(), "c64", "ntsc").as_deref(), Some("6510 · 1,023 MHz · NTSC · 263 Zeilen · 65 Zyklen/Zeile"));
        assert_eq!(takt_text(&data(), "c64", "gibtsnicht").as_deref(), takt_text(&data(), "c64", "pal").as_deref(), "unbekannte Norm = PAL");
    }
    #[test] fn nur_sichere_werte_mega65_ohne_zyklen_pet_ohne_zeilen_fehlendes_system_nichts() {
        let m = takt_text(&data(), "mega65", "pal").unwrap(); assert!(m.contains("45GS02") && m.contains("40,500 MHz") && !m.contains("Zyklen"), "MEGA65: Takt ja, Zyklen je Zeile nein: {m}");
        let p = takt_text(&data(), "pet", "pal").unwrap(); assert!(p.contains("6502") && !p.contains("Zeilen"), "PET: nur Takt: {p}");
        assert!(takt_text(&data(), "gibtsnicht", "pal").is_none());
        for s in ["atari8", "c128", "c16", "plus4", "vic20"] { let t = takt_text(&data(), s, "pal").unwrap(); assert!(t.contains("Zyklen/Zeile"), "{s}: {t}") }
    }
}
