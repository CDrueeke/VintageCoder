// SPDX-License-Identifier: GPL-3.0-or-later
//! „Auf Hardware hören“: ein Tracker-Lied als C64-Programm zusammensetzen, das die Ultimate über den echten SID spielt.
//!
//! Das Abspielprogramm besteht aus der Vorlage `data/templates/player/c64_sid.asm`, der Spielroutine `music.inc` und den Liedtabellen
//! (Export des Lieds als Assembler). Es wird im Speicher gebaut; im Projekt entsteht keine Datei.
use std::path::Path;

/// Warum ein Lied (Profil `profile`, `channels` Stimmen) nicht auf der Ultimate gespielt werden kann; `None` = möglich
pub fn unsupported_reason(profile: &str, chip: &str, channels: usize) -> Option<String> {
    match profile {
        "c64.sid.song" if chip == "sid" && channels == 3 => None,
        "c64.sid.song" => Some(format!("Für dieses Lied gibt es kein Abspielprogramm ({channels} Stimmen am Chip „{chip}“).")),
        "atari.pokey.song" => Some("Für POKEY-Lieder (Atari) gibt es noch kein Abspielprogramm für die Ultimate; sie spielen nur über den Synthesizer.".into()),
        "mega65.sid4.song" => Some("Das Lied hat zwölf Stimmen für den MEGA65; die Ultimate 64 spielt nur C64-Programme mit dem 3-stimmigen SID.".into()),
        "plus4.ted.song" => Some("Das Lied ist für den TED des Plus/4; die Ultimate 64 hat keinen TED.".into()),
        "vic20.vic.song" => Some("Das Lied ist für den VIC des VC 20; die Ultimate 64 hat diesen Klangchip nicht.".into()),
        p => Some(format!("Für das Profil „{p}“ gibt es kein Abspielprogramm.")),
    }
}

/// Setzt das Abspielprogramm (PRG mit BASIC-Starter) aus dem Text der Lied-Datei (`.vcsong`) zusammen. `ntsc` wählt die NTSC-Frequenztabelle.
/// Das Lied wird so genommen, wie es im Editor steht (auch ungespeichert).
pub fn build_player(data: &Path, song_json: &str, ntsc: bool) -> Result<Vec<u8>, String> {
    let media = vc_media::parse_media(song_json)?;
    let profiles = vc_media::load_profiles(data)?;
    let p = vc_media::profile(&profiles, &media.profile)?;
    let chip = p.raw["chip"].as_str().unwrap_or(""); let ch = p.raw["channels"].as_u64().unwrap_or(0) as usize;
    if let Some(r) = unsupported_reason(&p.id, chip, ch) { return Err(r); }
    // Export in einen eigenen Ordner, damit das Projekt unberührt bleibt
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let tmp = std::env::temp_dir().join(format!("vc-listen-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
    let result = (|| -> Result<Vec<u8>, String> {
        let mut v: serde_json::Value = serde_json::from_str(song_json).map_err(|e| e.to_string())?;
        v["name"] = serde_json::json!("song");
        std::fs::write(tmp.join("song.vcsong"), v.to_string()).map_err(|e| e.to_string())?;
        vc_media::project::export(&tmp, data, "song.vcsong", "asm", Some("song"), None, 10, 10).map_err(|e| format!("Lied: {e}"))?;
        let dir = data.join("templates/player");
        let src = std::fs::read_to_string(dir.join("c64_sid.asm")).map_err(|e| format!("Vorlage c64_sid.asm: {e}"))?;
        let cpu = vc_target::load_cpu(data, "6510")?;
        let mut o = vc_asm::Options::new(dir.join("c64_sid.asm").to_string_lossy(), vc_asm::Policy::Warn);
        o.include_dirs.push(tmp.join("build/export")); o.include_dirs.push(dir);
        if ntsc { o.defines.push(("NTSC".into(), 1)); }
        let out = vc_asm::assemble(&src, &cpu, &o).map_err(|d| format!("Abspielprogramm: {}", d.iter().find(|x| x.is_error()).map(|x| x.to_string()).unwrap_or_else(|| "Assemblerfehler".into())))?;
        vc_asm::to_prg(&out)
    })();
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

/// Lädt `prg` auf die Ultimate, startet es und öffnet den Audio-Strom, dessen Pakete in `hub` laufen. Blockiert (REST); nicht im Zeichenthread aufrufen.
/// Der Dienst gehört dem Aufrufer: [`StreamService::poll`] regelmäßig aufrufen; beim Verwerfen wird der Strom gestoppt.
pub fn start_hardware(host: &str, password: Option<&str>, opts: crate::streams::StreamOptions, prg: &[u8], hub: std::sync::Arc<std::sync::Mutex<vc_emu::streams::audio::AudioHub>>) -> Result<crate::streams::StreamService, String> {
    let mut u = vc_emu::Ultimate64::connect(host.trim(), password.filter(|p| !p.is_empty())).map_err(|e| format!("Die Ultimate ist nicht erreichbar: {e}"))?;
    if let Ok(mut h) = hub.lock() { h.player.reset(); h.packets = 0; }
    u.run_prg(prg)?;
    let mut svc = crate::streams::StreamService::new(host.trim(), password, opts);
    svc.start(vc_emu::streams::StreamKind::Audio, vc_emu::streams::audio::hub_sink(hub))?;
    Ok(svc)
}

/// Stoppt den Strom und setzt die Ultimate zurück (sonst spielt das Programm weiter). Fehler werden gemeldet, der Strom ist in jedem Fall gestoppt.
pub fn stop_hardware(svc: &mut crate::streams::StreamService, reset: bool) -> Result<(), String> {
    svc.stop_all();
    if !reset { return Ok(()); }
    let host = svc.host().to_string();
    vc_emu::Ultimate64::connect(&host, None).and_then(|mut u| u.reset()).map_err(|e| format!("Zurücksetzen der Ultimate: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { crate::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data/") }
    fn demo() -> String { std::fs::read_to_string(data().parent().unwrap().join("examples/c64/music_demo/music/tune.vcsong")).unwrap() }

    #[test]
    fn builds_a_player_prg_with_basic_stub() {
        let prg = build_player(&data(), &demo(), false).unwrap();
        assert_eq!(&prg[0..2], &[0x01, 0x08], "Ladeadresse $0801");
        assert!(crate::debug::sys_target(&prg).is_some(), "BASIC-Starter mit SYS");
        assert!(prg.len() > 600 && prg.len() < 20000, "{}", prg.len());
        let ntsc = build_player(&data(), &demo(), true).unwrap();
        assert_ne!(prg, ntsc, "andere Frequenztabelle");
    }
    #[test]
    fn reports_reasons_for_other_chips() {
        assert!(unsupported_reason("c64.sid.song", "sid", 3).is_none());
        for (p, c, n) in [("atari.pokey.song", "pokey", 4), ("mega65.sid4.song", "sid", 12), ("plus4.ted.song", "ted", 2), ("vic20.vic.song", "vic", 4), ("x", "y", 1)] { assert!(unsupported_reason(p, c, n).unwrap().len() > 20, "{p}"); }
    }
    #[test]
    fn refuses_unsupported_song_with_reason() {
        let mut v: serde_json::Value = serde_json::from_str(&demo()).unwrap(); v["profile"] = serde_json::json!("atari.pokey.song");
        let e = build_player(&data(), &v.to_string(), false).unwrap_err(); assert!(e.contains("POKEY"), "{e}");
        assert!(build_player(&data(), "kein json", false).is_err());
    }
}
