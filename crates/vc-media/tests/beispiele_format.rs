// SPDX-License-Identifier: GPL-3.0-or-later
//! Die Mediendateien der Beispiele liegen im Format, das die Anwendung beim Speichern schreibt: Öffnen und Speichern ohne Änderung verändert keine Datei (byte-gleich).
//! Gilt für die Dateien des Pixel-Editors (Sprites, Player/Missile, Zeichensätze, Kacheln); andere Editoren haben ihren eigenen Schreiber.
//! Mit `VC_BEISPIELE_NORMALISIEREN=1` schreibt der Test die abweichenden Dateien im App-Format neu.
use std::path::{Path, PathBuf};

fn sammeln(d: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(d) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { sammeln(&p, out); }
        else if p.extension().and_then(|x| x.to_str()).is_some_and(|x| ["vcsprite", "vcpmg", "vcchar", "vctiles"].contains(&x)) { out.push(p); }
    }
}

#[test]
fn beispiele_sind_im_speicherformat_der_anwendung() {
    let mut dateien = vec![]; sammeln(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples"), &mut dateien);
    assert!(!dateien.is_empty());
    let neu_schreiben = std::env::var("VC_BEISPIELE_NORMALISIEREN").is_ok();
    let mut abweichend = vec![];
    for p in dateien {
        let text = std::fs::read_to_string(&p).unwrap();
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
        if v.get("format").is_none() { continue; }   // keine Mediendatei im App-Format
        let soll = vc_media::project::format_media(&vc_media::romref::for_saving(&v));
        if soll != text { if neu_schreiben { std::fs::write(&p, soll).unwrap(); } abweichend.push(p.display().to_string()); }
    }
    assert!(neu_schreiben || abweichend.is_empty(), "nicht im Speicherformat: {abweichend:#?}");
}
