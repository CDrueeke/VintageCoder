// SPDX-License-Identifier: GPL-3.0-or-later
//! Klangprüfung ohne Audiogerät: Songs der Beispiele werden über `synth::Engine` in einen Puffer gerechnet; Kennwerte (Spitzenpegel, Effektivwert je halbe
//! Sekunde, Stille) werden gegen `klang_ref.json` geprüft, dazu die Grundfrequenz einzelner Töne gegen die Tonhöhentabelle.
//! Referenz neu schreiben (nur nach Freigabe): `VC_UPDATE_REFS=1 cargo test -p vc-ide-model --test klang`
use serde_json::{json, Value};
use std::path::PathBuf;
use vc_ide_model::songdoc::SongDoc;
use vc_ide_model::synth::{hz, Engine, Instr, SongData, Wave};

const SR: f32 = 44100.0;
fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }

fn rendern(song: SongData, sekunden: f32) -> Vec<f32> {
    let mut e = Engine::default(); e.play(song, 0, 0); let mut out = vec![0.0f32; (SR * sekunden) as usize];
    for stueck in out.chunks_mut(512) { e.render(stueck, SR) } out
}
fn kennwerte(p: &[f32]) -> Value {
    let spitze = p.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let fenster = (SR * 0.5) as usize; let rms: Vec<f64> = p.chunks(fenster).map(|c| (c.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / c.len() as f64).sqrt()).collect();
    json!({ "spitze": (spitze * 1000.0).round() / 1000.0, "rms_je_halbe_sekunde": rms.iter().map(|r| (r * 1000.0).round() / 1000.0).collect::<Vec<_>>() })
}
/// Grundfrequenz über Nulldurchgänge (steigend)
fn frequenz(p: &[f32]) -> f32 { let mut z = vec![]; for i in 1..p.len() { if p[i - 1] <= 0.0 && p[i] > 0.0 { z.push(i) } } if z.len() < 3 { return 0.0 } (z.len() - 1) as f32 * SR / (z[z.len() - 1] - z[0]) as f32 }

#[test]
fn beispiel_songs_klingen_nicht_still_und_nicht_uebersteuert_und_entsprechen_der_referenz() {
    let data = repo().join("data"); let (mut ist, ref_pfad) = (serde_json::Map::new(), PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/klang_ref.json"));
    for (name, bsp) in [("c64_tune", "c64/music_demo/music/tune.vcsong"), ("atari_pokey", "atari/music_demo/music/tune_pokey.vcsong"), ("mega65_tune4", "mega65/music_demo/music/tune4.vcsong")] {
        let dir = std::env::temp_dir().join(format!("vc-klang-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("m")).unwrap(); std::fs::copy(repo().join("examples").join(bsp), dir.join("m/s.vcsong")).unwrap();
        let s = SongDoc::oeffnen(&dir, "m/s.vcsong", &data).unwrap(); let p = rendern(s.song_daten(false), 4.0); let k = kennwerte(&p); let _ = std::fs::remove_dir_all(&dir);
        let spitze = k["spitze"].as_f64().unwrap(); assert!(spitze > 0.02, "{name}: Song ist still (Spitze {spitze})"); assert!(spitze <= 1.0, "{name}: übersteuert (Spitze {spitze})");
        assert!(k["rms_je_halbe_sekunde"].as_array().unwrap().iter().any(|r| r.as_f64().unwrap() > 0.005), "{name}: nie hörbar");
        ist.insert(name.into(), k);
    }
    if std::env::var_os("VC_UPDATE_REFS").is_some() { std::fs::write(&ref_pfad, serde_json::to_string_pretty(&Value::Object(ist)).unwrap() + "\n").unwrap(); return }
    let Ok(text) = std::fs::read_to_string(&ref_pfad) else { eprintln!("Klang: keine Referenz (übersprungen)"); return };
    let soll: Value = serde_json::from_str(&text).unwrap();
    for (name, k) in &ist {
        let r = &soll[name]; assert!(!r.is_null(), "{name}: nicht in der Referenz");
        let (a, b) = (k["spitze"].as_f64().unwrap(), r["spitze"].as_f64().unwrap()); assert!((a - b).abs() <= 0.02 + b * 0.05, "{name}: Spitzenpegel {a} statt {b}");
        for (i, (x, y)) in k["rms_je_halbe_sekunde"].as_array().unwrap().iter().zip(r["rms_je_halbe_sekunde"].as_array().unwrap()).enumerate() { let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap()); assert!((x - y).abs() <= 0.01 + y * 0.10, "{name}: Effektivwert in Halbsekunde {i}: {x} statt {y}") }
    }
}

#[test]
fn einzelne_toene_haben_die_richtige_grundfrequenz() {
    // Sägezahn und Rechteck, Töne A-2 … A-4 (Note 58 = 440 Hz laut Tabelle `hz`): Grundfrequenz auf 2 % genau
    for wave in [Wave::Saw, Wave::Pulse, Wave::Triangle] {
        for note in [34u8, 46, 58, 70] {
            let ins = Instr::Sid { wave, pw: 0.5, a: 0, d: 9, s: 15, r: 4 };
            let song = SongData { pokey: false, channels: 1, rows: 1, order: vec![0], loop_at: 0, patterns: vec![vec![vec![(note, 0)]]], instruments: vec![ins], row_secs: 2.0, frame_hz: 50.0 };
            let p = rendern(song, 1.0); let f = frequenz(&p[4410..]); let soll = hz(note);
            assert!((f - soll).abs() / soll < 0.02, "{wave:?} Note {note}: gemessen {f:.1} Hz, erwartet {soll:.1} Hz");
        }
    }
}

#[test]
fn stille_song_bleibt_still_und_notenende_beendet_den_ton() {
    let leer = SongData { pokey: false, channels: 1, rows: 4, order: vec![0], loop_at: 0, patterns: vec![vec![vec![(0, 0)]; 4]], instruments: vec![Instr::Sid { wave: Wave::Saw, pw: 0.5, a: 0, d: 9, s: 8, r: 4 }], row_secs: 0.1, frame_hz: 50.0 };
    assert!(rendern(leer, 1.0).iter().all(|x| x.abs() < 1e-4), "ohne Noten kein Ton");
    let ende = SongData { pokey: false, channels: 1, rows: 2, order: vec![0], loop_at: 0, patterns: vec![vec![vec![(58, 0)], vec![(255, 0)]]], instruments: vec![Instr::Sid { wave: Wave::Saw, pw: 0.5, a: 0, d: 9, s: 15, r: 0 }], row_secs: 0.2, frame_hz: 50.0 };
    let p = rendern(ende, 1.0); let rms = |a: f32, b: f32| { let (a, b) = ((SR * a) as usize, (SR * b) as usize); (p[a..b].iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / (b - a) as f64).sqrt() };
    assert!(rms(0.02, 0.18) > 0.01, "Ton in der ersten Zeile"); assert!(rms(0.25, 0.38) < 0.002, "nach dem Notenende (Release 0) still bis das Muster von vorn beginnt: {}", rms(0.25, 0.38));
}
