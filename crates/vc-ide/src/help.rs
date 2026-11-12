// SPDX-License-Identifier: GPL-3.0-or-later
//! Online-Hilfe: Handbuchthemen (Markdown, eingebettet), Nachschlagen eines Wortes im Kontext (Befehl, Direktive, Register, BASIC-Wort) und Suche.
use serde_json::{json, Value};
use std::path::Path;
use vc_target::{default_target_or_c64, load_chip, load_cpu, load_target, Cpu, Mode};

pub struct Topic { pub id: &'static str, pub group: &'static str, pub title: &'static str, pub body: &'static str }

pub const TOPICS: &[Topic] = &[
    Topic { id: "start", group: "Erste Schritte", title: "Überblick", body: include_str!("../help/start.md") },
    Topic { id: "shortcuts", group: "Erste Schritte", title: "Tastenkürzel", body: include_str!("../help/shortcuts.md") },
    Topic { id: "layout", group: "Erste Schritte", title: "Oberfläche anpassen", body: include_str!("../help/layout.md") },
    Topic { id: "settings", group: "Erste Schritte", title: "Einstellungen", body: include_str!("../help/settings.md") },
    Topic { id: "linux", group: "Erste Schritte", title: "Linux", body: include_str!("../help/linux.md") },
    Topic { id: "overview", group: "Erste Schritte", title: "Projekt-Übersicht", body: include_str!("../help/overview.md") },
    Topic { id: "about", group: "Erste Schritte", title: "Über VintageCoder", body: include_str!("../help/about.md") },
    Topic { id: "feedback", group: "Erste Schritte", title: "Feedback senden", body: include_str!("../help/feedback.md") },
    Topic { id: "editor", group: "Programmieren", title: "Editor", body: include_str!("../help/editor.md") },
    Topic { id: "assembler", group: "Programmieren", title: "Assembler", body: include_str!("../help/assembler.md") },
    Topic { id: "asm-expr", group: "Programmieren", title: "Assembler: Ausdrücke und Funktionen", body: include_str!("../help/asm-expr.md") },
    Topic { id: "asm-directives", group: "Programmieren", title: "Assembler: Direktiven", body: include_str!("../help/asm-directives.md") },
    Topic { id: "asm-text", group: "Programmieren", title: "Assembler: Texte und Zeichensätze", body: include_str!("../help/asm-text.md") },
    Topic { id: "asm-macros", group: "Programmieren", title: "Assembler: Labels, Makros, Scopes", body: include_str!("../help/asm-macros.md") },
    Topic { id: "asm-library", group: "Programmieren", title: "Assembler: Systembibliothek", body: include_str!("../help/asm-library.md") },
    Topic { id: "asm-index", group: "Programmieren", title: "Assembler: Spickzettel und Fehlercodes", body: include_str!("../help/asm-index.md") },
    Topic { id: "basic", group: "Programmieren", title: "BASIC", body: include_str!("../help/basic.md") },
    Topic { id: "basic-labels", group: "Programmieren", title: "BASIC: Sprungmarken und Zeilennummern", body: include_str!("../help/basic-labels.md") },
    Topic { id: "basic-names", group: "Programmieren", title: "BASIC: Lange Variablennamen", body: include_str!("../help/basic-names.md") },
    Topic { id: "basic-directives", group: "Programmieren", title: "BASIC: Direktiven, Dateien und Assets", body: include_str!("../help/basic-directives.md") },
    Topic { id: "basic-dialects", group: "Programmieren", title: "BASIC: Dialekte und Zielsysteme", body: include_str!("../help/basic-dialects.md") },
    Topic { id: "basic-tools", group: "Programmieren", title: "BASIC: Werkzeuge und Fehlersuche", body: include_str!("../help/basic-tools.md") },
    Topic { id: "basic-index", group: "Programmieren", title: "BASIC: Spickzettel und Fehlercodes", body: include_str!("../help/basic-index.md") },
    Topic { id: "build", group: "Programmieren", title: "Bauen und Starten", body: include_str!("../help/build.md") },
    Topic { id: "cli", group: "Programmieren", title: "Kommandozeile (vcc)", body: include_str!("../help/cli.md") },
    Topic { id: "debugger", group: "Testen", title: "Debugger", body: include_str!("../help/debugger.md") },
    Topic { id: "breakpoints", group: "Testen", title: "Haltepunkte", body: include_str!("../help/breakpoints.md") },
    Topic { id: "memmap", group: "Testen", title: "Memory Map", body: include_str!("../help/memmap.md") },
    Topic { id: "tests", group: "Testen", title: "Tests mit vcc test", body: include_str!("../help/tests.md") },
    Topic { id: "graphics", group: "Grafik und Klang", title: "Sprites, Zeichensätze, Songs", body: include_str!("../help/graphics.md") },
    Topic { id: "export", group: "Grafik und Klang", title: "Export", body: include_str!("../help/export.md") },
    Topic { id: "sprite", group: "Grafik und Klang", title: "Sprite-/PMG-Editor", body: include_str!("../help/sprite.md") },
    Topic { id: "charset", group: "Grafik und Klang", title: "Zeichensatz-Editor", body: include_str!("../help/charset.md") },
    Topic { id: "screen", group: "Grafik und Klang", title: "Bildschirm-Editor", body: include_str!("../help/screen.md") },
    Topic { id: "tiles", group: "Grafik und Klang", title: "Kachel-Editor", body: include_str!("../help/tiles.md") },
    Topic { id: "tracker", group: "Grafik und Klang", title: "Tracker", body: include_str!("../help/tracker.md") },
    Topic { id: "envelope", group: "Grafik und Klang", title: "Hüllkurve", body: include_str!("../help/envelope.md") },
    Topic { id: "cartridges", group: "Systeme und Formate", title: "Segmente, Bänke, Module", body: include_str!("../help/cartridges.md") },
    Topic { id: "disks", group: "Systeme und Formate", title: "Disk-Studio", body: include_str!("../help/disks.md") },
    Topic { id: "play", group: "Grafik und Klang", title: "PLAY-Tracker (MEGA65)", body: include_str!("../help/play.md") },
    Topic { id: "wizard", group: "Erste Schritte", title: "Willkommen und Projekt-Wizard", body: include_str!("../help/wizard.md") },
    Topic { id: "vic20", group: "Systeme und Formate", title: "VC-20", body: include_str!("../help/vic20.md") },
    Topic { id: "c128-plus4", group: "Systeme und Formate", title: "C128 und Plus/4", body: include_str!("../help/c128-plus4.md") },
    Topic { id: "pet-cbm2", group: "Systeme und Formate", title: "PET/CBM und CBM-II (Spike)", body: include_str!("../help/pet-cbm2.md") },
    Topic { id: "mega65", group: "Systeme und Formate", title: "MEGA65 (45GS02)", body: include_str!("../help/mega65.md") },
    Topic { id: "emulators", group: "Systeme und Formate", title: "Emulatoren (Homepage, portable Installation)", body: include_str!("../help/emulators.md") },
    Topic { id: "roms", group: "Systeme und Formate", title: "ROM-Dateien", body: include_str!("../help/roms.md") },
    Topic { id: "mega65-hardware", group: "Systeme und Formate", title: "MEGA65 am Gerät (USB, Ethernet; ungeprüft)", body: include_str!("../help/mega65-hardware.md") },
    Topic { id: "release-auswahl", group: "Für Entwickler", title: "Release-Auswahl (FEATURES.md, release.py)", body: include_str!("../help/release-auswahl.md") },
];

/// Mindestens eine dieser Kennungen (wie in `FEATURES.md`) muss im Release gewählt sein, damit das Thema erscheint; leer = immer.
/// Die Zuordnung steht auch in `release/registry.json` (Feld `help`); ein Test hält beide gleich.
pub fn needs(id: &str) -> &'static [&'static str] {
    const MEDIA: &[&str] = &["STU-SPRITE", "STU-CHAR", "STU-SCREEN", "STU-TILES", "STU-PALETTE", "STU-TRACKER", "STU-PLAY"];
    match id.strip_prefix("kapitel:").unwrap_or(id) {
        "sprite" | "sprites" => &["STU-SPRITE"],
        "charset" | "zeichensatz" => &["STU-CHAR"],
        "screen" => &["STU-SCREEN"],
        "tiles" | "bildschirm-kacheln" => &["STU-TILES", "STU-SCREEN"],
        "paletten" => &["STU-PALETTE"],
        "tracker" | "envelope" => &["STU-TRACKER"],
        "play" | "play-tracker" => &["STU-PLAY"],
        "disks" | "disk-studio" => &["STU-DISK"],
        "memmap" => &["STU-MEMMAP"],
        "graphics" | "export" | "grafik-ueberblick" => MEDIA,
        "vic20" => &["SYS-VC20"],
        "c128-plus4" => &["SYS-C128", "SYS-PLUS4"],
        "pet-cbm2" => &["SYS-PET", "SYS-CBM2"],
        "mega65" | "mega65-hardware" => &["SYS-MEGA65"],
        "emulators" => &["EMU-VICE", "EMU-ATARI800", "EMU-ALTIRRA", "EMU-FUJISAN", "EMU-XEMU"],
        _ => &[],
    }
}

/// Ist das Thema in dieser Auswahl sichtbar?
/// Die Seite „Release-Auswahl“ ist für Entwickler: sie erscheint nur im Quellbau (keine `release.json`, alles an), nicht in einem Release.
pub fn visible_with(id: &str, f: &crate::features::Features) -> bool { (id != "release-auswahl" || f.is_all()) && f.has_any(needs(id)) }
pub fn visible(id: &str) -> bool { visible_with(id, &crate::features::features()) }

pub fn topic_list() -> Value {
    let f = crate::features::features();
    let hb = handbook().iter().filter(|t| visible_with(t.id, &f)).map(|t| json!({ "id": t.id, "group": t.group, "title": t.title }));
    json!({ "topics": hb.chain(TOPICS.iter().filter(|t| visible_with(t.id, &f)).map(|t| json!({ "id": t.id, "group": t.group, "title": t.title }))).collect::<Vec<_>>() })
}

/// Kurzname eines Systems für die Seite „Emulatoren“
fn system_short(id: &str) -> String {
    match id { "c64" => "C64", "c128" => "C128", "plus4" => "Plus/4", "c16" => "C16/C116", "vic20" => "VC-20", "atari8" => "Atari 800 XL/XE", "mega65" => "MEGA65", other => other }.to_string()
}

fn mb(b: u64) -> String { format!("{:.1} MB", b as f64 / 1_048_576.0) }

/// Zeile je Betriebssystem eines Emulators: ladbare Pakete oder der Hinweis aus dem Katalog
fn os_lines(e: &crate::emulators::Entry) -> Vec<(String, String)> {
    ["windows", "linux", "macos"].iter().filter_map(|os| {
        let pk: Vec<String> = e.packages.iter().filter(|p| p.os == *os && p.sha256.len() == 64).map(|p| format!("{} {} ({})", p.version, if p.arch == "aarch64" { "ARM" } else { "64 Bit" }, mb(p.size))).collect();
        let text = if !pk.is_empty() { format!("portabel ladbar: {}", pk.join(", ")) } else { e.manual.get(*os).cloned().filter(|h| !h.is_empty())? };
        Some((match *os { "windows" => "Windows", "linux" => "Linux", _ => "macOS" }.to_string(), text))
    }).collect()
}

/// Tabelle „Übersicht“ der Seite „Emulatoren“ aus dem Katalog (nur gewählte Anbindungen)
fn emulators_table(es: &[&crate::emulators::Entry]) -> String {
    let mut t = String::from("| Emulator | Systeme | Rolle | Portabel installierbar |\n|---|---|---|---|\n");
    for e in es {
        let mut oses: Vec<&str> = ["windows", "linux", "macos"].iter().copied().filter(|os| e.packages.iter().any(|p| p.os == *os && p.sha256.len() == 64)).map(|o| match o { "windows" => "Windows", "linux" => "Linux", _ => "macOS" }).collect();
        oses.sort();
        let col = if oses.is_empty() { "–".to_string() } else { oses.join(", ") };
        t.push_str(&format!("| {} | {} | {} | {} |\n", e.name, e.systems.iter().map(|s| system_short(s)).collect::<Vec<_>>().join(", "), e.role, col));
    }
    t
}

/// Abschnitte `## <Name>` der Seite „Emulatoren“ aus dem Katalog (nur gewählte Anbindungen)
fn emulators_sections(es: &[&crate::emulators::Entry]) -> String {
    let mut out = String::new();
    for e in es {
        out.push_str(&format!("## {}\n\n{}\n\n- **Homepage:** [{h}]({h})\n- **Download-Seite:** [{d}]({d})\n- **Lizenz:** {}\n- **Systeme:** {}\n- **Rolle in VintageCoder:** {}\n", e.name, e.summary, e.license, e.systems.iter().map(|s| system_short(s)).collect::<Vec<_>>().join(", "), e.role, h = e.homepage, d = e.download_page));
        for (os, text) in os_lines(e) { out.push_str(&format!("- **{os}:** {text}\n")); }
        for n in &e.notes { out.push_str(&format!("- {n}\n")); }
        out.push('\n');
    }
    out
}

/// Seite „Emulatoren“: die Vorlage mit Tabelle und Abschnitten aus dem Emulator-Katalog; Emulatoren, deren Anbindung das Release nicht enthält, fehlen
fn emulators_page(md: &str) -> String { emulators_page_in(md, &crate::features::features()) }

fn emulators_page_in(md: &str, f: &crate::features::Features) -> String {
    let es = crate::emulators::available_in(f);
    md.replace("<!-- EMULATOR-TABLE -->\n", &emulators_table(&es)).replace("<!-- EMULATOR-SECTIONS -->\n", &emulators_sections(&es))
}

/// Thema nach Kennung; Handbuchkapitel heißen `kapitel:<id>`
pub fn topic(id: &str) -> Option<Value> {
    let id = id.strip_prefix("hilfe:").or_else(|| id.strip_prefix("help:")).unwrap_or(id);
    handbook().iter().chain(TOPICS.iter()).find(|t| t.id == id && visible(t.id)).map(|t| json!({ "kind": "topic", "id": t.id, "group": t.group, "title": t.title, "markdown": if t.id == "emulators" { emulators_page(t.body) } else { t.body.to_string() } }))
}

// ---- Handbuch: Kapitel aus docs/handbuch/kapitel (eingebettet), Kopf (YAML) ausgewertet ----

/// Gruppenname der Handbuchkapitel in der Hilfe
pub const HANDBOOK_GROUP: &str = "Handbuch";

const CHAPTERS: &[&str] = &[
    include_str!("../../../docs/handbuch/kapitel/00-installation.md"),
    include_str!("../../../docs/handbuch/kapitel/01-willkommen.md"),
    include_str!("../../../docs/handbuch/kapitel/02-erstes-projekt.md"),
    include_str!("../../../docs/handbuch/kapitel/03-oberflaeche.md"),
    include_str!("../../../docs/handbuch/kapitel/04-projekte.md"),
    include_str!("../../../docs/handbuch/kapitel/05-editor.md"),
    include_str!("../../../docs/handbuch/kapitel/06-assembler.md"),
    include_str!("../../../docs/handbuch/kapitel/07-basic.md"),
    include_str!("../../../docs/handbuch/kapitel/08-bauen-starten.md"),
    include_str!("../../../docs/handbuch/kapitel/09-debugger.md"),
    include_str!("../../../docs/handbuch/kapitel/10-tests.md"),
    include_str!("../../../docs/handbuch/kapitel/11-grafik-ueberblick.md"),
    include_str!("../../../docs/handbuch/kapitel/12-zeichensatz.md"),
    include_str!("../../../docs/handbuch/kapitel/13-sprites.md"),
    include_str!("../../../docs/handbuch/kapitel/14-bildschirm-kacheln.md"),
    include_str!("../../../docs/handbuch/kapitel/15-paletten.md"),
    include_str!("../../../docs/handbuch/kapitel/16-tracker.md"),
    include_str!("../../../docs/handbuch/kapitel/17-play-tracker.md"),
    include_str!("../../../docs/handbuch/kapitel/18-disk-studio.md"),
    include_str!("../../../docs/handbuch/kapitel/19-systeme.md"),
    include_str!("../../../docs/handbuch/kapitel/20-kuerzel-faq.md"),
];

/// Bilder des Handbuchs (Kennung, Alt-Text, Unterschrift); Bilder, die hier fehlen, sind noch nicht aufgenommen
const IMAGE_MANIFEST: &str = include_str!("../../../docs/handbuch/bilder/manifest.json");

/// Kopf eines Handbuchkapitels
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChapterMeta { pub id: String, pub title: String, pub order: i32, pub level: String, pub time: String, pub see_also: Vec<String>, pub reference: Vec<String> }

/// Trennt den YAML-Kopf (`---` … `---`) vom Text. Ohne Kopf oder ohne `id`/`titel`: `None`.
pub fn parse_frontmatter(src: &str) -> Option<(ChapterMeta, &str)> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let rest = src.strip_prefix("---\n").or_else(|| src.strip_prefix("---\r\n"))?;
    let mut end = None; let mut pos = 0;
    for line in rest.split_inclusive('\n') { if line.trim_end() == "---" { end = Some((pos, pos + line.len())); break; } pos += line.len(); }
    let (head_end, body_start) = end?;
    let mut m = ChapterMeta::default();
    for line in rest[..head_end].lines() {
        let Some((k, v)) = line.split_once(':') else { continue };
        let v = v.trim();
        let list = || v.trim_start_matches('[').trim_end_matches(']').split(',').map(|x| x.trim().trim_matches('"').to_string()).filter(|x| !x.is_empty()).collect::<Vec<_>>();
        match k.trim() {
            "id" => m.id = v.to_string(), "titel" => m.title = v.trim_matches('"').to_string(), "reihenfolge" => m.order = v.parse().unwrap_or(0),
            "stufe" => m.level = v.to_string(), "zeit" => m.time = v.to_string(), "siehe_auch" => m.see_also = list(), "referenz" => m.reference = list(),
            _ => {}
        }
    }
    if m.id.is_empty() || m.title.is_empty() { return None; }
    Some((m, rest[body_start..].trim_start_matches(['\r', '\n'])))
}

/// Kennungen der Bilder, die im Handbuch vorhanden (aufgenommen) sind
pub fn handbook_images() -> &'static [(String, String, String)] {
    static I: std::sync::OnceLock<Vec<(String, String, String)>> = std::sync::OnceLock::new();
    I.get_or_init(|| {
        let v: Value = serde_json::from_str(IMAGE_MANIFEST).unwrap_or(Value::Null);
        v["bilder"].as_array().map(|a| a.iter().filter_map(|b| Some((b["id"].as_str()?.to_string(), b["alt"].as_str().unwrap_or("").to_string(), b["caption"].as_str().unwrap_or("").to_string()))).collect()).unwrap_or_default()
    })
}

/// Gibt es das Bild `id` (Datei `<id>.png`)? Unbekannte Kennungen lässt die Hilfe weg.
pub fn handbook_image_known(id: &str) -> bool { handbook_images().iter().any(|(i, _, _)| i == id) }

/// Handbuchkapitel als Hilfethemen, nach `reihenfolge` sortiert; Kennung `kapitel:<id>`, Gruppe „Handbuch“
pub fn handbook() -> &'static [Topic] {
    static H: std::sync::OnceLock<Vec<Topic>> = std::sync::OnceLock::new();
    H.get_or_init(|| {
        let mut ch: Vec<(ChapterMeta, &'static str)> = CHAPTERS.iter().filter_map(|c| parse_frontmatter(c)).collect();
        ch.sort_by_key(|(m, _)| m.order);
        let title_of = |id: &str| ch.iter().find(|(m, _)| m.id == id).map(|(m, _)| m.title.clone());
        ch.iter().map(|(m, body)| {
            let (first, rest) = body.split_once('\n').unwrap_or((body, ""));
            let mut md = format!("{first}\n\n*Stufe: {} · Zeit: {}*\n{rest}", m.level, m.time);
            let see: Vec<String> = m.see_also.iter().filter_map(|i| title_of(i).map(|t| format!("[{t}](kapitel:{i})"))).collect();
            if !see.is_empty() { md.push_str(&format!("\n\n**Siehe auch im Handbuch:** {}", see.join(" · "))); }
            let refs: Vec<String> = m.reference.iter().filter_map(|i| TOPICS.iter().find(|t| t.id == i).map(|t| format!("[{}](hilfe:{i})", t.title))).collect();
            if !refs.is_empty() { md.push_str(&format!("\n\n**Zum Nachschlagen:** {}", refs.join(" · "))); }
            md.push('\n');
            Topic { id: Box::leak(format!("kapitel:{}", m.id).into_boxed_str()), group: HANDBOOK_GROUP, title: Box::leak(m.title.clone().into_boxed_str()), body: Box::leak(md.into_boxed_str()) }
        }).collect()
    })
}

fn mode_syntax(m: Mode) -> &'static str {
    match m {
        Mode::Imp => "", Mode::Acc => "A", Mode::Imm => "#$nn", Mode::Imw => "#$nnnn", Mode::Zp => "$nn", Mode::Zpx => "$nn,X", Mode::Zpy => "$nn,Y", Mode::Abs => "$nnnn",
        Mode::Abx => "$nnnn,X", Mode::Aby => "$nnnn,Y", Mode::Ind => "($nnnn)", Mode::Izx => "($nn,X)", Mode::Izy => "($nn),Y", Mode::Rel => "Ziel", Mode::Rl => "Ziel (16 Bit)",
        Mode::Izz => "($nn),Z", Mode::Isy => "($nn,SP),Y", Mode::Iax => "($nnnn,X)", Mode::Zpr => "$nn,Ziel",
    }
}
fn mode_name(m: Mode) -> &'static str {
    match m {
        Mode::Imp => "implizit", Mode::Acc => "Akkumulator", Mode::Imm => "unmittelbar", Mode::Imw => "unmittelbar 16 Bit", Mode::Zp => "Zeropage", Mode::Zpx => "Zeropage,X", Mode::Zpy => "Zeropage,Y",
        Mode::Abs => "absolut", Mode::Abx => "absolut,X", Mode::Aby => "absolut,Y", Mode::Ind => "indirekt", Mode::Izx => "(Zeropage,X)", Mode::Izy => "(Zeropage),Y", Mode::Rel => "relativ",
        Mode::Rl => "relativ 16 Bit", Mode::Izz => "(Zeropage),Z", Mode::Isy => "(Stack,SP),Y", Mode::Iax => "(absolut,X)", Mode::Zpr => "Zeropage + relativ",
    }
}

fn target_cpu(data: &Path, target: &str) -> Result<(vc_target::Target, Cpu), String> {
    let t = load_target(data, &if target.is_empty() { default_target_or_c64(Some(data)) } else { target.to_string() })?;
    let cpu = load_cpu(data, &t.cpu)?;
    Ok((t, cpu))
}

fn mnemonic_page(cpu: &Cpu, word: &str) -> Option<Value> {
    let w = word.to_ascii_uppercase();
    let w = if w == "NOP" && !cpu.has_mnemonic("NOP") && cpu.has_mnemonic("EOM") { "EOM".to_string() } else if w == "CMPQ" { "CPQ".to_string() } else { w };
    let (base, quad) = match cpu.quad_base(&w) { Some(b) => (b.to_string(), true), None => (w.clone(), false) };
    if !cpu.has_mnemonic(&base) { return None; }
    let mut ops: Vec<&vc_target::Op> = cpu.ops.iter().filter(|o| o.mnemonic == base).collect();
    if quad { ops.retain(|o| matches!(o.mode, Mode::Imp | Mode::Acc | Mode::Zp | Mode::Zpx | Mode::Abs | Mode::Abx | Mode::Izz)); }
    ops.sort_by_key(|o| o.opcode);
    let doc = vc_lsp::mnemonic_doc(&base).unwrap_or("");
    let mut md = format!("# {w}\n\n{doc}\n\n");
    if quad { md.push_str(&format!("Q-Befehl der 45GS02: Präfix `NEG NEG` (`$42 $42`) vor dem Opcode von `{base}`; wirkt auf das 32-Bit-Register Q (A, X, Y, Z). Mit `[$nn],z` zusätzlich Präfix `$EA` (32-Bit-Zeiger).\n\n")); }
    if ops.iter().any(|o| o.mode == Mode::Izz) && !quad { md.push_str("Mit `[$nn],z` (Präfix `$EA`) wird der Zeiger 32 Bit breit.\n\n"); }
    md.push_str("| Adressierung | Schreibweise | Opcode | Bytes | Zyklen | Flags |\n|---|---|---|---|---|---|\n");
    let mut any_illegal = false;
    for o in &ops {
        let syn = mode_syntax(o.mode); any_illegal |= o.group != "official";
        let pre = if quad { 2 + (o.mode == Mode::Izz) as u8 * 0 } else { 0 };
        md.push_str(&format!("| {} | `{} {}` | `{}${:02X}` | {} | {}{}{} | {} |\n", mode_name(o.mode), w.to_ascii_lowercase(), syn.to_ascii_lowercase(), if quad { "42 42 " } else { "" }, o.opcode, o.bytes + pre, o.cycles,
            if o.page_cross { "+1" } else { "" }, if o.branch { "+1/+2" } else { "" }, if o.affects.is_empty() { "–".into() } else { o.affects.clone() }));
    }
    if any_illegal { md.push_str("\nEinige Formen sind **undokumentiert** (`illegals = allow|warn` im `.cpu`-Kopf nötig).\n"); }
    md.push_str("\nZyklenangaben: `+1` bei Seitenüberschreitung, `+1/+2` bei genommenem Sprung. Flags: N=Negativ, V=Überlauf, Z=Null, C=Übertrag.\n");
    Some(json!({ "kind": "mnemonic", "id": w, "title": w, "markdown": md, "topics": ["assembler"] }))
}

fn register_pages(data: &Path, t: &vc_target::Target, word: &str) -> Vec<(String, String, u32, String, String)> {
    // (Chip, Register, Adresse, Zugriff, Beschreibung); `chip.reg` oder nur der Registername
    let (chip_part, reg_part) = match word.split_once('.') { Some((c, r)) => (Some(c), r), None => (None, word) };
    let mut out = vec![];
    for c in &t.chips {
        let Ok(chip) = load_chip(data, c) else { continue };
        if chip_part.map_or(false, |p| p != chip.id) { continue; }
        for r in &chip.regs { if r.name == reg_part { out.push((chip.id.clone(), r.name.clone(), r.addr, r.access.clone(), r.doc.clone())); } }
    }
    out
}

/// Hilfe zu einem Wort im Kontext von `target` und `lang` (`asm` oder `basic`)
pub fn lookup(data: &Path, target: &str, word: &str, lang: &str) -> Result<Value, String> {
    let raw = word.trim();
    let w = raw.trim_matches(|c: char| matches!(c, ',' | ':' | ';' | '(' | ')' | '#' | '"' | '[' | ']')).to_ascii_lowercase();
    if w.is_empty() { return Ok(json!({ "kind": "none", "title": "", "markdown": "" })); }
    if lang == "basic" {
        let text = vc_core::read_static(&data.join("help/basic_keywords.json")).map_err(|e| format!("basic_keywords.json: {e}"))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let key = w.to_ascii_uppercase();
        if let Some(k) = v["keywords"].get(&key).or_else(|| v["keywords"].get(format!("{key}$"))) {
            let md = format!("# {key}\n\n{}\n\n```\n{}\n```\n\nSiehe [BASIC](help:basic).", k["text"].as_str().unwrap_or(""), k["example"].as_str().unwrap_or(""));
            return Ok(json!({ "kind": "keyword", "id": key, "title": key, "markdown": md, "topics": ["basic"] }));
        }
    }
    if w.starts_with('.') {
        if let Some(d) = vc_lsp::directive_doc(&w) {
            let rel = if matches!(w.as_str(), ".bank" | ".segment" | ".cartridge") { "cartridges" } else { "assembler" };
            return Ok(json!({ "kind": "directive", "id": w, "title": w, "markdown": format!("# {w}\n\n{d}\n\nSiehe [{}](help:{rel}).", if rel == "cartridges" { "Segmente, Bänke, Module" } else { "Assembler" }), "topics": [rel] }));
        }
    }
    if let Ok((t, cpu)) = target_cpu(data, target) {
        if let Some(p) = mnemonic_page(&cpu, &w) { return Ok(p); }
        let regs = register_pages(data, &t, &w);
        if !regs.is_empty() {
            let mut md = format!("# {}\n\n", if w.contains('.') { w.clone() } else { format!("Register {w}") });
            for (c, n, a, acc, d) in &regs { md.push_str(&format!("**{c}.{n}** – Adresse `${a:04X}`, Zugriff {acc}\n\n{d}\n\n")); }
            md.push_str("Im Quelltext als `chip.register` verwendbar (z. B. `lda vic2.border`).");
            return Ok(json!({ "kind": "register", "id": w, "title": w, "markdown": md, "topics": ["assembler"] }));
        }
    }
    let syms = crate::sysmap::symbols(data, target);
    if let Some(sym) = crate::sysmap::find(&syms, &w) {
        return Ok(json!({ "kind": "symbol", "id": sym.name, "title": sym.name, "markdown": crate::sysmap::page(&syms, sym), "topics": ["assembler"] }));
    }
    if let Some(t) = TOPICS.iter().find(|t| (t.id == w || t.title.to_lowercase() == w) && visible(t.id)) { return Ok(topic(t.id).unwrap()); }
    Ok(json!({ "kind": "none", "title": w, "markdown": format!("Zu „{raw}“ habe ich keinen Eintrag. Die Suche findet Themen, Befehle, Direktiven und Register.") }))
}

fn snippet(body: &str, q: &str) -> String {
    let low = body.to_lowercase();
    match low.find(q) {
        Some(i) => { let start = body[..i].rfind('\n').map_or(0, |p| p + 1); let end = body[i..].find('\n').map_or(body.len(), |p| i + p); body[start..end].trim().chars().take(140).collect() }
        None => body.lines().find(|l| !l.trim().is_empty() && !l.starts_with('#')).unwrap_or("").chars().take(140).collect(),
    }
}

/// Suche über Themen, Befehle, Direktiven, Register und BASIC-Wörter
pub fn search(data: &Path, target: &str, q: &str) -> Result<Value, String> {
    let q = q.trim().to_lowercase();
    if q.is_empty() { return Ok(json!({ "results": [] })); }
    let mut res: Vec<(i32, Value)> = vec![];
    for t in handbook().iter().chain(TOPICS.iter()).filter(|t| visible(t.id)) {
        let (title, body) = (t.title.to_lowercase(), t.body.to_lowercase());
        let n = body.matches(&q).count() as i32;
        let score = if title.contains(&q) { 50 } else { 0 } + n.min(10) * 2 + if t.id == q { 60 } else { 0 };
        if score > 0 { res.push((score, json!({ "kind": "topic", "id": t.id, "title": t.title, "snippet": snippet(t.body, &q) }))); }
    }
    for (d, doc) in vc_lsp::DIRECTIVES { if d.contains(&q) || (q.len() > 2 && doc.to_lowercase().contains(&q)) { res.push((if d.trim_start_matches('.') == q.trim_start_matches('.') { 70 } else { 20 }, json!({ "kind": "directive", "id": d, "title": d, "snippet": doc }))); } }
    if let Ok((t, cpu)) = target_cpu(data, target) {
        let mut names: Vec<String> = cpu.ops.iter().map(|o| o.mnemonic.to_lowercase()).chain(cpu.quad_ops.iter().map(|(q, _)| q.to_lowercase())).collect(); names.sort(); names.dedup();
        for n in names.iter().filter(|n| n.starts_with(&q)).take(8) { res.push((if *n == q { 80 } else { 15 }, json!({ "kind": "mnemonic", "id": n, "title": n.to_uppercase(), "snippet": vc_lsp::mnemonic_doc(&n.to_uppercase()).unwrap_or("") }))); }
        let mut seen = 0;
        'outer: for c in &t.chips { if let Ok(chip) = load_chip(data, c) { for r in &chip.regs { if r.name.contains(&q) || format!("{}.{}", chip.id, r.name).contains(&q) { res.push((if r.name == q { 40 } else { 10 }, json!({ "kind": "register", "id": format!("{}.{}", chip.id, r.name), "title": format!("{}.{}", chip.id, r.name), "snippet": format!("${:04X} {}", r.addr, r.doc) }))); seen += 1; if seen >= 10 { break 'outer; } } } } }
    }
    if let Ok(text) = vc_core::read_static(&data.join("help/basic_keywords.json")) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(o) = v["keywords"].as_object() { for (k, e) in o { if k.to_lowercase().starts_with(&q) { res.push((if k.to_lowercase() == q { 60 } else { 12 }, json!({ "kind": "keyword", "id": k, "title": k, "snippet": e["text"].as_str().unwrap_or("") }))); } } }
        }
    }
    for sym in crate::sysmap::symbols(data, target).iter().filter(|s| s.name.to_lowercase().contains(&q) || (q.len() > 3 && s.doc.to_lowercase().contains(&q))).take(12) {
        res.push((if sym.name.to_lowercase() == q { 75 } else if sym.name.to_lowercase().starts_with(&q) { 14 } else { 8 }, json!({ "kind": "symbol", "id": sym.name, "title": sym.name, "snippet": format!("${:04X}  {}", sym.addr, sym.doc) })));
    }
    res.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(json!({ "results": res.into_iter().take(40).map(|r| r.1).collect::<Vec<_>>() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_needs_stimmt() {
        let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../release/registry.json")).expect("registry.json");
        let v: Value = serde_json::from_str(&text).unwrap();
        for (key, f) in v["features"].as_object().unwrap() {
            for t in f["help"].as_array().into_iter().flatten().filter_map(|x| x.as_str()) {
                assert!(TOPICS.iter().any(|x| x.id == t) || handbook().iter().any(|x| x.id == format!("kapitel:{t}")), "{key}: Hilfethema {t} gibt es nicht");
                assert!(needs(t).contains(&key.as_str()), "{key}: needs({t}) = {:?}", needs(t));
            }
        }
    }

    #[test]
    fn abgewaehltes_studio_blendet_hilfe_aus() {
        use crate::features::Features;
        let nur_sprite = Features::from_json(r#"{"format":1,"studios":["sprite"],"targets":["c64"]}"#);
        assert!(visible_with("sprite", &nur_sprite) && !visible_with("tracker", &nur_sprite) && !visible_with("kapitel:tracker", &nur_sprite));
        assert!(visible_with("graphics", &nur_sprite), "Überblick bleibt, solange ein Medien-Studio da ist");
        assert!(!visible_with("graphics", &Features::from_json(r#"{"format":1,"studios":["disk"]}"#)));
        assert!(!visible_with("mega65", &nur_sprite) && visible_with("editor", &nur_sprite));
        assert!(visible_with("tracker", &Features::all()));
    }

    #[test]
    fn links_zwischen_themen_fuehren_zu_vorhandenen_themen() {
        for t in TOPICS {
            for part in t.body.split("](help:").skip(1) {
                let id: String = part.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
                assert!(TOPICS.iter().any(|x| x.id == id), "Thema {}: Link auf unbekanntes Thema „{id}“", t.id);
            }
        }
    }

    #[test]
    fn themen_sind_eindeutig_und_nicht_leer() {
        let mut ids: Vec<&str> = TOPICS.iter().map(|t| t.id).collect();
        ids.sort(); ids.dedup();
        assert_eq!(ids.len(), TOPICS.len());
        assert!(TOPICS.iter().all(|t| t.body.starts_with("# ") && t.body.len() > 100), "jede Seite beginnt mit einer Überschrift");
        for id in ["settings", "overview", "memmap", "export", "tests", "disks", "wizard"] { assert!(topic(id).is_some(), "{id}"); }
    }

    #[test]
    fn emulatorseite_nennt_alle_emulatoren_des_katalogs() {
        let md = topic("emulators").unwrap()["markdown"].as_str().unwrap().to_string();
        assert!(!md.contains("EMULATOR-"), "Platzhalter nicht ersetzt");
        for e in crate::emulators::catalog() {
            assert!(md.contains(&format!("## {}", e.name)), "Abschnitt für {} fehlt (help/emulators.md neu erzeugen)", e.id);
            assert!(md.contains(&e.homepage) && md.contains(&e.download_page) && md.contains(&e.license), "{}: Homepage, Download-Seite oder Lizenz fehlt", e.id);
            for p in e.packages.iter().filter(|p| p.sha256.len() == 64) { assert!(md.contains(&p.version), "{}: Version {} fehlt", e.id, p.version); }
        }
    }

    #[test]
    fn emulatorseite_zeigt_nur_gewaehlte() {
        let f = crate::features::Features::from_json(r#"{"format":1,"links":["vice"]}"#);
        let md = emulators_page_in(include_str!("../help/emulators.md"), &f);
        assert!(md.contains("## VICE") && md.contains("| VICE |"), "{md}");
        for n in ["atari800", "Altirra", "Xemu", "Fujisan"] { assert!(!md.contains(&format!("## {n}")) && !md.contains(&format!("| {n}")), "{n}"); }
        assert!(md.contains("## Portabel installieren") && !md.contains("EMULATOR-"));
    }

    fn links_of<'a>(body: &'a str, scheme: &str) -> Vec<String> {
        body.split(&format!("]({scheme}:")[..]).skip(1).map(|p| p.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect()).collect()
    }

    #[test]
    fn frontmatter_wird_ausgewertet_und_entfernt() {
        let src = "---\nid: demo\ntitel: Ein Titel: mit Doppelpunkt\nreihenfolge: 7\nstufe: Einsteiger\nzeit: 5 Minuten\nsiehe_auch: [a, b]\nreferenz: [start]\n---\n\n# Ein Titel\n\nText\n";
        let (m, body) = parse_frontmatter(src).unwrap();
        assert_eq!((m.id.as_str(), m.title.as_str(), m.order, m.level.as_str(), m.time.as_str()), ("demo", "Ein Titel: mit Doppelpunkt", 7, "Einsteiger", "5 Minuten"));
        assert_eq!((m.see_also, m.reference), (vec!["a".to_string(), "b".into()], vec!["start".to_string()]));
        assert!(body.starts_with("# Ein Titel"), "{body:?}");
        assert!(parse_frontmatter("# Nur Text\n").is_none());
        assert!(parse_frontmatter("---\nid: x\n# nie geschlossen\n").is_none());
        assert!(parse_frontmatter("---\ntitel: ohne id\n---\n# T\n").is_none());
    }

    #[test]
    fn handbuch_ist_sortiert_ohne_kopf_und_in_der_gruppe() {
        let hb = handbook();
        assert_eq!(hb.len(), CHAPTERS.len(), "jedes Kapitel hat einen gültigen Kopf");
        assert!(hb.iter().all(|t| t.group == HANDBOOK_GROUP && t.id.starts_with("kapitel:") && t.body.starts_with("# ") && !t.body.contains("\nreihenfolge:")));
        assert_eq!(hb[0].id, "kapitel:installation");
        assert_eq!(hb[1].id, "kapitel:willkommen");
        let mut ids: Vec<&str> = hb.iter().map(|t| t.id).collect(); ids.sort(); ids.dedup();
        assert_eq!(ids.len(), hb.len());
        let l = topic_list(); assert_eq!(l["topics"][0]["group"], HANDBOOK_GROUP);
        assert_eq!(topic("kapitel:editor").unwrap()["title"], "Der Code-Editor");
        assert!(topic("kapitel:gibtsnicht").is_none());
        assert!(topic("hilfe:start").is_some() && topic("help:start").is_some());
    }

    #[test]
    fn handbuchlinks_fuehren_zu_vorhandenen_seiten() {
        for t in handbook() {
            for id in links_of(t.body, "kapitel") { assert!(topic(&format!("kapitel:{id}")).is_some(), "{}: Link auf unbekanntes Kapitel „{id}“", t.id); }
            for scheme in ["hilfe", "help"] { for id in links_of(t.body, scheme) { assert!(TOPICS.iter().any(|x| x.id == id), "{}: Link auf unbekanntes Hilfethema „{id}“", t.id); } }
        }
    }

    #[test]
    fn handbuchbilder_sind_bekannt_oder_fehlen_noch() {
        let mut fehlen = std::collections::BTreeSet::new();
        for t in handbook() { for id in links_of(t.body, "bild") { if !handbook_image_known(&id) { fehlen.insert(id); } } }
        // fehlende Kennungen (noch nicht aufgenommen) sind erlaubt, die Hilfe lässt sie weg; die Liste soll aber nicht unbemerkt wachsen
        assert!(fehlen.len() <= 12, "viele Bilder ohne Eintrag im Manifest: {fehlen:?}");
        for (id, alt, _) in handbook_images() { assert!(!id.is_empty() && !alt.is_empty()); assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../docs/handbuch/bilder/{id}.png")).is_file(), "{id}.png fehlt"); }
    }

    #[test]
    fn suche_findet_handbuchtexte() {
        let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let r = search(&data, "c64", "Rinne").unwrap();
        assert!(r["results"].as_array().unwrap().iter().any(|x| x["id"].as_str().unwrap_or("").starts_with("kapitel:")), "{r}");
        let r = search(&data, "c64", "Willkommen bei VintageCoder").unwrap();
        assert!(r["results"].as_array().unwrap().iter().any(|x| x["id"] == "kapitel:willkommen"), "{r}");
    }

    #[test]
    fn release_seite_nennt_alle_unterbefehle_und_die_umgebungsvariable() {
        let seite = topic("release-auswahl").unwrap()["markdown"].as_str().unwrap().to_string();
        // Unterbefehle aus dem Skript selbst lesen (add_parser), damit ein neuer Unterbefehl ohne Hilfetext auffällt
        let py = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/release.py")).unwrap();
        let mut namen: Vec<String> = py.lines().filter_map(|l| {
            let i = l.find("add_parser(\"")? + "add_parser(\"".len();
            Some(l[i..].split('"').next()?.to_string())
        }).collect();
        namen.sort();
        namen.dedup();
        for n in ["check", "plan", "stage", "bootstrap"] { assert!(namen.iter().any(|x| x == n), "tools/release.py kennt den Unterbefehl {n} nicht mehr"); }
        for n in &namen { assert!(seite.contains(&format!("tools/release.py {n}")), "help/release-auswahl.md nennt `release.py {n}` nicht"); }
        assert!(seite.contains("VC_RELEASE"), "Umgebungsvariable VC_RELEASE fehlt auf der Seite");
        // jede Kennung aus FEATURES.md steht auf der Seite
        let features = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../FEATURES.md")).unwrap();
        // HTML-Kommentare (Zeilenmuster im Kopf der Datei) zählen nicht
        let features = match (features.find("<!--"), features.find("-->")) { (Some(i), Some(j)) if i < j => format!("{}{}", &features[..i], &features[j + 3..]), _ => features };
        let mut n = 0;
        for l in features.lines().filter(|l| l.starts_with("| `")) {
            let k = l[3..].split('`').next().unwrap();
            assert!(seite.contains(&format!("`{k}`")), "help/release-auswahl.md nennt `{k}` nicht");
            n += 1;
        }
        assert!(n >= 25);
    }

    #[test]
    fn release_seite_nur_im_quellbau() {
        assert!(visible_with("release-auswahl", &crate::features::Features::all()));
        let f = crate::features::Features::from_json(r#"{"format":1,"studios":["sprite"]}"#);
        assert!(!visible_with("release-auswahl", &f) && visible_with("start", &f));
    }

    #[test]
    fn kommandozeilenseite_nennt_alle_befehle() {
        let cli = topic("cli").unwrap()["markdown"].as_str().unwrap().to_string();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/cli");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().map_or(true, |x| x != "json") || p.file_name().unwrap() == "_vcc.json" { continue; }
            let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
            let name = v["command"].as_str().unwrap();
            assert!(cli.contains(&format!("`vcc {name}`")), "help/cli.md nennt `vcc {name}` nicht; Seite neu erzeugen/ergänzen");
            n += 1;
        }
        assert!(n >= 10);
        let v: Value = serde_json::from_str(&std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/cli/_vcc.json")).unwrap()).unwrap();
        for e in v["env"].as_array().unwrap() { assert!(cli.contains(e["name"].as_str().unwrap()), "Umgebungsvariable fehlt auf der Seite"); }
    }
}
