// Bilder für Handbuch und Hilfe aus der neuen Oberfläche (wird in `mod tests` eingebunden).
// `cargo test -p vc-gui handbuch_bilder -- --ignored` schreibt nach `build/handbuch/<kennung>.png` (dunkel, 1600×1007, Beispielprojekt „Hugo & Gumbo“
// oder ein anderes Beispiel, ohne Marken); nur Auswahl: `VC_ZUSTAENDE=editor-suchen,asm-fehler`. Nichts Persönliches: neutraler Projektordner und
// Speicherort; Debug-Bilder zeigen eine Attrappe (es läuft kein Emulator).

/// Debug-Attrappe über dem Quelltext von „Hugo & Gumbo“: angehalten in `main` (Zeile 61), ein Haltepunkt auf Zeile 60
fn hb_attrappe(a: &mut App) {
    a.oeffne("jumprun.asm"); if let Some(e) = a.ed() { e.lines.truncate(70); e.version += 1; e.breakpoints.insert(59); }
    let dl = |addr: u16, b: &[u8], t: &str| debug::DisLine { addr, bytes: b.to_vec(), text: t.into(), illegal: false, cycles: 3, extra: false };
    a.info = Some(BauInfo { target: "c64".into(), artifact: a.root.join("build/jumprun.prg"), symbols: [("main".to_string(), 0x0820i64), ("start".to_string(), 0x0815i64)].into(), run: Some(0x0815), segmente: vec![(0x0801, 0x400)],
        zeilen: vec![("jumprun.asm".into(), 59, 0x0820, 2), ("jumprun.asm".into(), 60, 0x0822, 3), ("jumprun.asm".into(), 61, 0x0825, 3), ("jumprun.asm".into(), 62, 0x0828, 3), ("jumprun.asm".into(), 63, 0x082B, 3)] });
    let snap = debug::Snap { have_regs: true, running: false, pc: 0x0825, a: 0x00, x: 0x28, y: 0x00, sp: 0xF6, flags: 0x26, disasm: vec![dl(0x0820, &[0xA9, 0x00], "lda #$00"), dl(0x0822, &[0x8D, 0x4A, 0x08], "sta $084a"), dl(0x0825, &[0x8D, 0x4B, 0x08], "sta $084b"), dl(0x0828, &[0x8D, 0x4C, 0x08], "sta $084c"), dl(0x082B, &[0x20, 0x70, 0x08], "jsr $0870")],
        mem_addr: 0x0820, mem: (0..=255u8).map(|i| i.wrapping_mul(7).wrapping_add(0x20)).collect(), watch: vec![("$d020".into(), Ok(0)), ("$084b".into(), Ok(0)), ("cur".into(), Ok(0))],
        detected: Some(Ok(vc_ide::snapshot::Detected { system: "c64".into(), supported: true, mode: "Text".into(), screen: 0x0400, rom_off: Some(0), ..Default::default() })),
        points: vec![debug::PointInfo { id: 1, kind: "exec".into(), start: 0x0822, end: 0x0822, hits: 1, ..Default::default() }], ..Default::default() };
    a.dbg = Some(debug::Debugger::demo(snap)); a.beob = vec!["$d020".into(), "$084b".into(), "cur".into()];
    let _ = a.update(Msg::DebugTick); a.layout.debug_start(); a.schublade = 3;
}

/// Alle Szenen: Kennung, Beispielprojekt, Aufbau
fn hb_szenen() -> Vec<(&'static str, &'static str, Box<dyn Fn(&mut App)>)> {
    const J: &str = "examples/c64/jumprun";
    // Nur für das Bild: Datei im Editor auf die ersten Zeilen kürzen (nichts wird gespeichert)
    let kurz = |a: &mut App, n: usize| { if let Some(e) = a.ed() { e.lines.truncate(n); e.version += 1; if n >= 70 { e.scroll_row = 44.0 } } };
    let mut v: Vec<(&'static str, &'static str, Box<dyn Fn(&mut App)>)> = vec![];
    v.push(("editor-suchen", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 32); let _ = a.befehl(Cmd::Suchen); let _ = a.update(Msg::FindFrage("sta".into())); })));
    v.push(("editor-ersetzen", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 32); let _ = a.befehl(Cmd::Ersetzen); let _ = a.update(Msg::FindFrage("sc".into())); a.find.mit = "punkte".into(); })));
    v.push(("editor-gehe-zu", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 32); let _ = a.befehl(Cmd::GeheZuZeile); a.goto.text = "59".into(); })));
    v.push(("editor-kontextmenue", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); if let Some(e) = a.ed() { e.anchor = Some(vc_gui_editor::core::Pos { line: 59, col: 8 }); e.cursor = vc_gui_editor::core::Pos { line: 59, col: 14 } } let _ = a.update(Msg::KontextAuf(Art::Editor, iced::Point::new(520.0, 380.0), None)); })));
    v.push(("editor-falten", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); if let Some(e) = a.ed() { e.toggle_fold(47) } })));
    v.push(("editor-zweiter-bereich", J, Box::new(move |a| { a.oeffne("jumprun.asm"); a.oeffne("lib/music.inc"); kurz(a, 30); a.aktiv = 0; kurz(a, 30); let _ = a.befehl(Cmd::Teilen); })));
    v.push(("editor-fehlerzeile", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); if let Some(e) = a.ed() { e.lines[60] = "        jmp ende".into(); e.errors.insert(60, "E202: unbekanntes Symbol „ende“".into()); e.version += 1 } })));
    v.push(("datei-neu-dialog", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 30); let _ = a.update(Msg::BaumNeu(String::new(), NeuArt::Asm)); })));
    v.push(("datei-umbenennen-dialog", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 30); a.baum_sel = Some("hal_c64.asm".into()); let _ = a.befehl(Cmd::DateiUmbenennen); })));
    v.push(("drag-drop-rueckfrage", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 30); let ext = std::path::PathBuf::from("/tmp/vc-handbuch/ablage"); std::fs::create_dir_all(&ext).unwrap(); std::fs::write(ext.join("level5.asm"), "nop\n").unwrap(); std::fs::write(ext.join("hugo_springt.vcsprite"), "{}").unwrap(); let _ = a.update(Msg::Abgelegt(ext.join("level5.asm"))); let _ = a.update(Msg::Abgelegt(ext.join("hugo_springt.vcsprite"))); })));
    v.push(("projekt-einstellungen", J, Box::new(|a| { let _ = a.befehl(Cmd::ProjektEinstellungen); })));
    v.push(("bauen-knopf", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); })));
    v.push(("probleme-panel", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); if let Some(e) = a.ed() { e.lines[60] = "        jmp ende".into(); e.errors.insert(60, "E202: unbekanntes Symbol „ende“".into()); e.cursor = vc_gui_editor::core::Pos { line: 60, col: 8 } } a.probleme.push(Problem { datei: "jumprun.asm".into(), zeile: 61, text: "E202: unbekanntes Symbol „ende“".into(), fehler: true }); let _ = a.update(Msg::Schublade(1)); })));
    v.push(("ausgabe-panel", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); a.log = vec!["Projekt geöffnet: Hugo & Gumbo".into(), "Bauen: jumprun.asm …".into(), "Fertig: 5823 Bytes → build/jumprun.prg".into(), "Gestartet: x64sc".into()]; let _ = a.update(Msg::Schublade(0)); })));
    v.push(("tests-panel", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); use vc_ide::tests::suite::{Event, Group, Item}; use vc_ide::tests::{Level, Outcome}; let fl = std::path::PathBuf::from("tests/spielablauf.vctest"); a.tests.suite.apply(Event::Static(Group { title: "Statische Prüfungen".into(), items: vec![Item { file: fl.clone(), outcome: Outcome::passed("keine Überlappung", None) }, Item { file: fl.clone(), outcome: Outcome::failed("Programm kleiner als $2000", Level::Error, "erwartet ≤ $2000, ist $2310") }], ..Default::default() })); let _ = a.update(Msg::Schublade(2)); })));
    v.push(("tests-ergebnis", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); use vc_ide::tests::suite::{Event, Group, Item}; use vc_ide::tests::{Level, Outcome}; let fl = std::path::PathBuf::from("tests/spielablauf.vctest"); a.tests.suite.apply(Event::Static(Group { title: "Statische Prüfungen".into(), items: vec![Item { file: fl.clone(), outcome: Outcome::passed("keine Überlappung", None) }, Item { file: fl.clone(), outcome: Outcome::failed("Programm kleiner als $2000", Level::Error, "erwartet ≤ $2000, ist $2310") }], ..Default::default() })); let _ = a.update(Msg::Schublade(2)); let _ = a.update(Msg::TestWahl(fl, "Programm kleiner als $2000".into())); })));
    v.push(("memory-map", J, Box::new(|a| { hb_attrappe(a); let _ = a.update(Msg::PanelWahl(PanelSeite::Karte)); a.layout.aufklappen(Bereich::Panel); })));
    // Studios und Dialoge (Beispielprojekte der Auslieferung)
    v.push(("zeichensatz-multicolor", J, Box::new(|a| { a.oeffne("graphics/chars.vcchar"); let n = a.name().unwrap(); a.studio_aktion(&n, studio::Aktion::Frame(3)); })));
    v.push(("zeichensatz-standard", J, Box::new(|a| { a.oeffne("graphics/chars.vcchar"); let _ = a.befehl(Cmd::StdZeichensatz); })));
    v.push(("zeichensatz-png-import", J, Box::new(|a| { a.oeffne("graphics/chars.vcchar"); let _ = a.befehl(Cmd::ZeichensatzAusPng); })));
    v.push(("sprite-animation", J, Box::new(|a| { a.oeffne("graphics/hero_over.vcsprite"); let rel = a.name().unwrap(); use vc_ide_model::pixel::AnimOp; for op in [AnimOp::New, AnimOp::AddStep(1), AnimOp::AddStep(2), AnimOp::AddStep(3), AnimOp::Rename("laufen".into()), AnimOp::Delay(0, 4), AnimOp::Delay(1, 4), AnimOp::Delay(2, 6)] { let _ = a.update(Msg::StudioAkt(rel.clone(), studio::Aktion::Anim(op))); } })));
    v.push(("sprite-atari-pmg", "examples/atari/pmg_ship", Box::new(|a| { a.oeffne("graphics/ship.vcpmg"); })));
    v.push(("palette-mega65", "examples/mega65/pal_demo", Box::new(|a| { a.oeffne("graphics/pal65.vcpal"); })));
    v.push(("tracker-huellkurve", "examples/c64/music_demo", Box::new(|a| { a.oeffne("music/tune.vcsong"); let rel = a.name().unwrap(); let _ = a.update(Msg::SongAkt(rel, songstudio::GAkt::Modus(songstudio::SongModus::Instrumente))); })));
    v.push(("tracker-pokey", "examples/atari/music_demo", Box::new(|a| { a.oeffne("music/tune_pokey.vcsong"); let rel = a.name().unwrap(); let _ = a.update(Msg::SongAkt(rel, songstudio::GAkt::Modus(songstudio::SongModus::Instrumente))); })));
    v.push(("tracker-hoerprobe", J, Box::new(|a| { a.oeffne("music/tune.vcsong"); let rel = a.name().unwrap(); a.songs.get_mut(&rel).unwrap().klingt = vec![49, 53, 56, 70]; a.einst.tastaturlayout = einstellungen::Tastaturlayout::Qwertz; })));
    v.push(("play-tracker", "examples/mega65/play_demo", Box::new(|a| { a.oeffne("music/tune_play.vcplay"); })));
    v.push(("disk-neu", J, Box::new(|a| { a.oeffne("jumprun.asm"); let _ = a.update(Msg::BaumNeu(String::new(), NeuArt::Diskette("d64"))); })));
    v.push(("system-vergleich-c64", "examples/c64/border", Box::new(|a| { a.oeffne("border.asm"); })));
    v.push(("system-vergleich-atari", "examples/atari/colorbars", Box::new(|a| { a.oeffne("colorbars.asm"); })));
    v.push(("system-vergleich-mega65", "examples/mega65/hello", Box::new(|a| { a.oeffne("hello.asm"); })));
    v.push(("einstellungen-emulator", J, Box::new(|a| { let _ = a.befehl(Cmd::Einstellungen); a.satz.seite = setui::Seite::Ziele; })));
    v.push(("einstellungen-roms", J, Box::new(|a| { let roms = a.root.join("roms"); std::fs::create_dir_all(roms.join("c64")).unwrap(); a.satz.roms.test_ordner = Some(roms); let _ = a.befehl(Cmd::Einstellungen); a.satz.seite = setui::Seite::Roms; let st = vc_roms::RomStore::new(a.rom_env()); a.satz.roms.stand = Some(std::sync::Arc::new(st)); })));
    v.push(("willkommen-seite", J, Box::new(|a| { a.tabs.clear(); a.start.recent = vec![vc_ide_model::recent::Recent { path: "~/Projekte/Hugo-Gumbo".into(), name: "Hugo & Gumbo".into(), pinned: true, last: vc_ide_model::recent::now() - 3600, missing: false, info: vc_ide_model::recent::Info { systems: vec![("c64".into(), "pal".into())], langs: vec!["vcasm".into()], emulator: Some("VICE x64sc".into()), ..Default::default() } }, vc_ide_model::recent::Recent { path: "~/Projekte/Hallo".into(), name: "Hallo".into(), pinned: false, last: vc_ide_model::recent::now() - 86400 * 3, missing: false, info: vc_ide_model::recent::Info { systems: vec![("mega65".into(), String::new())], langs: vec!["vcasm".into()], ..Default::default() } }]; let _ = a.befehl(Cmd::Startseite); })));
    v.push(("assistent-neues-projekt", J, Box::new(|a| { a.oeffne("jumprun.asm"); let _ = a.befehl(Cmd::ProjektNeu); if let Some(n) = a.neu_projekt.as_mut() { n.ort = std::path::PathBuf::from("Projekte"); n.name = "hallo".into() } })));
    // Quelltext und Projekte
    v.push(("editor-vervollstaendigung", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); if let Some(e) = a.ed() { e.cursor = vc_gui_editor::core::Pos { line: 74, col: 16 }; e.insert("\n        sta vic2.b"); e.update_completion_ab(&["vic2.border", "vic2.bg0", "vic2.bg1", "vic2.bg2", "vic2.bg3"], 2); } })));
    v.push(("editor-hilfe-f1", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); if let Some(e) = a.ed() { e.cursor = vc_gui_editor::core::Pos { line: 59, col: 10 } } let _ = a.befehl(Cmd::Hilfe); })));
    v.push(("include-ziehen", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); if let Some(e) = a.ed() { e.einfuegemarke = Some(52) } })));
    v.push(("asm-makro", J, Box::new(move |a| { a.oeffne("lib/music.inc"); if let Some(e) = a.ed() { e.lines.truncate(52); e.version += 1; e.scroll_row = 17.0 } })));
    v.push(("basic-marken", "examples/c64/basic_native", Box::new(|a| { a.oeffne("sterne.bas"); if let Some(e) = a.ed() { e.lines.truncate(50); e.version += 1; e.scroll_row = 15.0 } })));
    v.push(("basic-langenamen", "examples/c64/basic_langenamen", Box::new(|a| { a.oeffne("punkte.bas"); if let Some(e) = a.ed() { e.lines.truncate(50); e.version += 1; e.scroll_row = 13.0 } })));
    v.push(("projekt-neu-ergebnis", J, Box::new(|a| { let ort = std::path::PathBuf::from("/tmp/vc-handbuch/neu"); let _ = std::fs::remove_dir_all(&ort); std::fs::create_dir_all(&ort).unwrap(); let _ = a.befehl(Cmd::ProjektNeu); if let Some(n) = a.neu_projekt.as_mut() { n.ort = ort; n.name = "hallo".into() } let _ = a.update(Msg::NeuProj(neuprojekt::NpAkt::Ok)); a.log = vec!["Projekt geöffnet: hallo".into()]; })));
    v.push(("emulator-reiter", J, Box::new(|a| { hb_attrappe(a); let _ = a.update(Msg::PanelWahl(PanelSeite::Emulator)); a.layout.aufklappen(Bereich::Panel); })));
    v.push(("tracker-sid-pokey", J, Box::new(|a| { a.oeffne("music/tune.vcsong"); let fremd = a.root.join("atari-lied.vcsong"); let p = vc_ide_model::songimport::konvertieren(&a.root, &a.data, &a.root.join("music/tune.vcsong"), "atari.pokey.song", "f", "music", 0, 30).unwrap(); std::fs::rename(a.root.join(p.0), &fremd).unwrap(); a.song_import_oeffnen("music/tune.vcsong", false, fremd); })));
    // Datenträger: eine Diskette mit neutralen Beispieldateien
    let disk = |a: &mut App| { let mut d = vc_disk::D64::blank("HUGO GUMBO", "01").unwrap(); for (n, g) in [("HUGO GUMBO", 5835usize), ("TITEL", 2400), ("BILD 1 WIESE", 1100), ("BILD 2 HOEHLE", 1100), ("BILD 3 BURG", 1100), ("BILD 4 WOLKEN", 1100), ("MUSIK", 3400), ("SPRITES", 3700)] { d.add_prg(n, &vec![(n.len() as u8).wrapping_mul(9); g]).unwrap() } std::fs::write(a.root.join("hugo-gumbo.d64"), d.to_bytes()).unwrap(); a.projekt_neu_laden(); a.oeffne("hugo-gumbo.d64"); };
    v.push(("disk-studio", J, Box::new(move |a| { disk(a); let _ = a.update(Msg::DiskAkt("hugo-gumbo.d64".into(), diskstudio::DAkt::Datei(1))); let _ = a.update(Msg::DiskAkt("hugo-gumbo.d64".into(), diskstudio::DAkt::KetteAn(true))); })));
    v.push(("disk-validieren", J, Box::new(move |a| { disk(a); let _ = a.update(Msg::DiskAkt("hugo-gumbo.d64".into(), diskstudio::DAkt::Validieren)); })));
    v.push(("argumente", J, Box::new(|a| { let _ = a.befehl(Cmd::Einstellungen); a.satz.seite = setui::Seite::Ziele; if let Some(eid) = a.satz.entwurf.target("c64").and_then(|t| t.emulators.first().map(|e| e.id.clone())) { let _ = a.update(Msg::Set(setui::SetAkt::Arg(argsui::ArgAkt::Auf(format!("args:{eid}"), false)))); } })));
    // Menüleiste im Fenster (so sieht sie unter Windows und Linux aus; unter macOS steht sie in der Menüleiste des Systems)
    v.push(("menueleiste", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); a.menue_test = true; a.kuerzel_windows = true; a.menue_oeffnen("Bearbeiten"); })));
    // Nur für die Website (Ausgabe `website_bilder_nachtrag`)
    v.push(("web-zeichen-c64", J, Box::new(move |a| { a.oeffne("jumprun.asm"); kurz(a, 75); let _ = a.update(Msg::PanelWahl(PanelSeite::Zeichen)); a.layout.aufklappen(Bereich::Panel); })));
    v.push(("web-zeichen-atari", "examples/atari/jumprun", Box::new(|a| { a.oeffne("jumprun.asm"); let _ = a.update(Msg::PanelWahl(PanelSeite::Zeichen)); a.layout.aufklappen(Bereich::Panel); })));
    v.push(("web-editor-atari", "examples/atari/colorbars", Box::new(|a| { a.oeffne("colorbars.asm"); })));
    v.push(("web-editor-mega65", "examples/mega65/hello", Box::new(|a| { a.oeffne("hello.asm"); })));
    v.push(("web-tracker-mega65", "examples/mega65/music_demo", Box::new(|a| { a.oeffne("music/tune4.vcsong"); for c in 6..12 { let _ = a.update(Msg::SongAkt("music/tune4.vcsong".into(), songstudio::GAkt::Sichtbar(c))); } })));
    v.push(("web-turbobasic-atari", "examples/atari/turbo_basic", Box::new(|a| { let n = a.namen.iter().find(|n| n.ends_with(".tur") || n.ends_with(".bas") || n.ends_with(".sbas")).cloned().expect("Turbo-BASIC-Quelltext"); a.oeffne(&n); })));
    v.push(("debug-ueberblick", J, Box::new(|a| hb_attrappe(a))));
    v
}

/// Szene für Szene nach `build/handbuch/` (dunkel, 1600×1007)
#[test] #[ignore] fn handbuch_bilder() {
    let aus = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/handbuch"); std::fs::create_dir_all(&aus).unwrap();
    let wahl = std::env::var("VC_ZUSTAENDE").unwrap_or_default();
    for (name, beispiel, richte) in hb_szenen() {
        if !wahl.is_empty() && !wahl.split(',').any(|w| w == name) { continue }
        let roh = kopie_von_beispiel(beispiel); let tmp = std::path::PathBuf::from("/tmp/vc-handbuch").join(format!("{}", std::process::id())).join(if beispiel.ends_with("jumprun") { "Hugo-Gumbo" } else { beispiel.rsplit('/').next().unwrap() }); let _ = std::fs::remove_dir_all(tmp.parent().unwrap()); std::fs::create_dir_all(tmp.parent().unwrap()).unwrap(); std::fs::rename(&roh, &tmp).unwrap(); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.einst.zoom = 1.0; app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false;
        richte(&mut app); if app.root == tmp && beispiel.ends_with("jumprun") { app.proj.name = "Hugo & Gumbo".into() }
        // Rest: lange Quelltexte kürzen, damit keine halbe Zeile über die Schublade ragt (nur Bild, nichts wird gespeichert)
        for e in app.editoren.values_mut() { if e.scroll_row == 0.0 && e.lines.len() > 33 { e.lines.truncate(33); e.version += 1 } }
        app.einst.schema = Schema::Dunkel; let (b, h) = match name { "sprite-animation" => (1600, 1300), "argumente" => (1600, 1500), _ => (1600, 1007) }; foto_gross(&mut app, &[], aus.join(format!("{name}.png")).to_str().unwrap(), b, h);
        let _ = std::fs::remove_dir_all(tmp.parent().unwrap());
    }
}

/// Nachtrag für die Website (dunkel und hell, 1600×1007) nach `build/web/`: Datenträger, Paletten-Studio, Debugger (Attrappe) und Menüleiste im Fenster
#[test] #[ignore] fn website_bilder_nachtrag() {
    let aus = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/web"); std::fs::create_dir_all(&aus).unwrap();
    for (web, szene) in [("studio-disk", "disk-studio"), ("studio-palette", "palette-mega65"), ("debugger", "debug-ueberblick"), ("menueleiste", "menueleiste"), ("zeichen-c64", "web-zeichen-c64"), ("zeichen-atari", "web-zeichen-atari"), ("editor-atari", "web-editor-atari"), ("editor-mega65", "web-editor-mega65"), ("tracker-mega65", "web-tracker-mega65")] {
        let (_, beispiel, richte) = hb_szenen().into_iter().find(|(n, _, _)| *n == szene).expect("Szene");
        let roh = kopie_von_beispiel(beispiel); let tmp = std::path::PathBuf::from("/tmp/vc-handbuch").join(format!("w{}", std::process::id())).join(if beispiel.ends_with("jumprun") { "Hugo-Gumbo" } else { beispiel.rsplit('/').next().unwrap() }); let _ = std::fs::remove_dir_all(tmp.parent().unwrap()); std::fs::create_dir_all(tmp.parent().unwrap()).unwrap(); std::fs::rename(&roh, &tmp).unwrap(); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.einst.zoom = 1.0; app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false;
        richte(&mut app); if app.root == tmp && beispiel.ends_with("jumprun") { app.proj.name = "Hugo & Gumbo".into() }
        for e in app.editoren.values_mut() { if e.scroll_row == 0.0 && e.lines.len() > 33 { e.lines.truncate(33); e.version += 1 } }
        for (schema, sn) in [(Schema::Dunkel, "dunkel"), (Schema::Hell, "hell")] { app.einst.schema = schema; foto_gross(&mut app, &[], aus.join(format!("{web}-{sn}.png")).to_str().unwrap(), 1600, 1007) }
        let _ = std::fs::remove_dir_all(tmp.parent().unwrap());
    }
}
