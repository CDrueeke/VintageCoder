// SPDX-License-Identifier: GPL-3.0-or-later
//! Online-Hilfe: Handbuchthemen (Markdown, eingebettet), Nachschlagen eines Wortes im Kontext (Befehl, Direktive, Register, BASIC-Wort) und Suche.
use serde_json::{json, Value};
use std::path::Path;
use vc_target::{default_target_or_c64, load_chip, load_cpu, load_target, Cpu, Mode};

pub struct Topic { pub id: &'static str, pub group: &'static str, pub title: &'static str, pub body: &'static str }

pub const TOPICS: &[Topic] = &[
    Topic { id: "start", group: "Erste Schritte", title: "Überblick", body: include_str!("../help/start.md") },
    Topic { id: "shortcuts", group: "Erste Schritte", title: "Tastenkürzel", body: include_str!("../help/shortcuts.md") },
    Topic { id: "filetree", group: "Erste Schritte", title: "Dateibaum: Marken und Farben", body: include_str!("../help/dateibaum.md") },
    Topic { id: "layout", group: "Erste Schritte", title: "Oberfläche anpassen", body: include_str!("../help/layout.md") },
    Topic { id: "paths", group: "Erste Schritte", title: "Pfadfelder und „Wählen …“", body: include_str!("../help/pfadfelder.md") },
    Topic { id: "markdown", group: "Erste Schritte", title: "Markdown-Dateien und Vorschau", body: include_str!("../help/markdown.md") },
    Topic { id: "numbers", group: "Erste Schritte", title: "Zahlenwerkzeuge", body: include_str!("../help/zahlenwerkzeuge.md") },
    Topic { id: "basic2asm", group: "Erste Schritte", title: "BASIC in Assembler umwandeln", body: include_str!("../help/basic-nach-asm.md") },
    Topic { id: "config", group: "Erste Schritte", title: "Konfiguration sichern und zurücksetzen", body: include_str!("../help/konfiguration.md") },
    Topic { id: "clock", group: "Erste Schritte", title: "Takt in der Statusleiste", body: include_str!("../help/takt-anzeige.md") },
    Topic { id: "settings", group: "Erste Schritte", title: "Einstellungen", body: include_str!("../help/settings.md") },
    Topic { id: "linux", group: "Erste Schritte", title: "Linux", body: include_str!("../help/linux.md") },
    Topic { id: "overview", group: "Erste Schritte", title: "Projekt-Übersicht", body: include_str!("../help/overview.md") },
    Topic { id: "about", group: "Erste Schritte", title: "Über VintageCoder", body: include_str!("../help/about.md") },
    Topic { id: "license", group: "Erste Schritte", title: "Lizenz und Danksagungen", body: include_str!("../help/license.md") },
    Topic { id: "feedback", group: "Erste Schritte", title: "Feedback senden", body: include_str!("../help/feedback.md") },
    Topic { id: "updates", group: "Erste Schritte", title: "Nach Updates suchen", body: include_str!("../help/updates.md") },
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
    Topic { id: "license-build", group: "Programmieren", title: "Lizenzhinweis beim Übersetzen", body: include_str!("../help/license-build.md") },
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
    Topic { id: "disks", group: "Systeme und Formate", title: "Disk-Editor", body: include_str!("../help/disks.md") },
    Topic { id: "play", group: "Grafik und Klang", title: "PLAY-Tracker (MEGA65)", body: include_str!("../help/play.md") },
    Topic { id: "wizard", group: "Erste Schritte", title: "Startseite und Projekt-Assistent", body: include_str!("../help/wizard.md") },
    Topic { id: "vic20", group: "Systeme und Formate", title: "VC-20", body: include_str!("../help/vic20.md") },
    Topic { id: "c128-plus4", group: "Systeme und Formate", title: "C128 und Plus/4", body: include_str!("../help/c128-plus4.md") },
    Topic { id: "pet", group: "Systeme und Formate", title: "PET/CBM (Spike)", body: include_str!("../help/pet.md") },
    Topic { id: "mega65", group: "Systeme und Formate", title: "MEGA65 (45GS02)", body: include_str!("../help/mega65.md") },
    Topic { id: "emulators", group: "Systeme und Formate", title: "Emulatoren (Homepage, portable Installation)", body: include_str!("../help/emulators.md") },
    Topic { id: "roms", group: "Systeme und Formate", title: "ROM-Dateien", body: include_str!("../help/roms.md") },
    Topic { id: "mega65-hardware", group: "Systeme und Formate", title: "MEGA65 am Gerät (USB, Ethernet; ungeprüft)", body: include_str!("../help/mega65-hardware.md") },
    Topic { id: "turbo-disk", group: "Systeme und Formate", title: "Turbo BASIC XL: Diskette", body: include_str!("../help/turbo-basic-diskette.md") },
    Topic { id: "extensions", group: "Für Entwickler", title: "Erweiterungen: Sicherheit und Rechte", body: include_str!("../help/extensions-security.md") },
    Topic { id: "release-auswahl", group: "Für Entwickler", title: "Release-Auswahl (FEATURES.md, release.py)", body: include_str!("../help/release-auswahl.md") },
];

/// Englische Fassungen der Hilfeseiten (`help/en/<datei>`), nach Themenkennung; fehlt eine, gilt die deutsche
const EN_BODIES: &[(&str, &str)] = &[
    ("start", include_str!("../help/en/start.md")),
    ("shortcuts", include_str!("../help/en/shortcuts.md")),
    ("filetree", include_str!("../help/en/dateibaum.md")),
    ("layout", include_str!("../help/en/layout.md")),
    ("paths", include_str!("../help/en/pfadfelder.md")),
    ("markdown", include_str!("../help/en/markdown.md")),
    ("numbers", include_str!("../help/en/zahlenwerkzeuge.md")),
    ("basic2asm", include_str!("../help/en/basic-nach-asm.md")),
    ("config", include_str!("../help/en/konfiguration.md")),
    ("clock", include_str!("../help/en/takt-anzeige.md")),
    ("settings", include_str!("../help/en/settings.md")),
    ("linux", include_str!("../help/en/linux.md")),
    ("overview", include_str!("../help/en/overview.md")),
    ("about", include_str!("../help/en/about.md")),
    ("license", include_str!("../help/en/license.md")),
    ("feedback", include_str!("../help/en/feedback.md")),
    ("updates", include_str!("../help/en/updates.md")),
    ("editor", include_str!("../help/en/editor.md")),
    ("assembler", include_str!("../help/en/assembler.md")),
    ("asm-expr", include_str!("../help/en/asm-expr.md")),
    ("asm-directives", include_str!("../help/en/asm-directives.md")),
    ("asm-text", include_str!("../help/en/asm-text.md")),
    ("asm-macros", include_str!("../help/en/asm-macros.md")),
    ("asm-library", include_str!("../help/en/asm-library.md")),
    ("asm-index", include_str!("../help/en/asm-index.md")),
    ("basic", include_str!("../help/en/basic.md")),
    ("basic-labels", include_str!("../help/en/basic-labels.md")),
    ("basic-names", include_str!("../help/en/basic-names.md")),
    ("basic-directives", include_str!("../help/en/basic-directives.md")),
    ("basic-dialects", include_str!("../help/en/basic-dialects.md")),
    ("basic-tools", include_str!("../help/en/basic-tools.md")),
    ("basic-index", include_str!("../help/en/basic-index.md")),
    ("license-build", include_str!("../help/en/license-build.md")),
    ("build", include_str!("../help/en/build.md")),
    ("cli", include_str!("../help/en/cli.md")),
    ("debugger", include_str!("../help/en/debugger.md")),
    ("breakpoints", include_str!("../help/en/breakpoints.md")),
    ("memmap", include_str!("../help/en/memmap.md")),
    ("tests", include_str!("../help/en/tests.md")),
    ("graphics", include_str!("../help/en/graphics.md")),
    ("export", include_str!("../help/en/export.md")),
    ("sprite", include_str!("../help/en/sprite.md")),
    ("charset", include_str!("../help/en/charset.md")),
    ("screen", include_str!("../help/en/screen.md")),
    ("tiles", include_str!("../help/en/tiles.md")),
    ("tracker", include_str!("../help/en/tracker.md")),
    ("envelope", include_str!("../help/en/envelope.md")),
    ("cartridges", include_str!("../help/en/cartridges.md")),
    ("disks", include_str!("../help/en/disks.md")),
    ("play", include_str!("../help/en/play.md")),
    ("wizard", include_str!("../help/en/wizard.md")),
    ("vic20", include_str!("../help/en/vic20.md")),
    ("c128-plus4", include_str!("../help/en/c128-plus4.md")),
    ("pet", include_str!("../help/en/pet.md")),
    ("mega65", include_str!("../help/en/mega65.md")),
    ("emulators", include_str!("../help/en/emulators.md")),
    ("roms", include_str!("../help/en/roms.md")),
    ("mega65-hardware", include_str!("../help/en/mega65-hardware.md")),
    ("turbo-disk", include_str!("../help/en/turbo-basic-diskette.md")),
    ("extensions", include_str!("../help/en/extensions-security.md")),
    ("release-auswahl", include_str!("../help/en/release-auswahl.md")),
];
/// Titel der Themen und Namen der Gruppen auf Englisch (`help/en/_index.json`, Schlüssel `titles`, `groups`)
const EN_INDEX: &str = include_str!("../help/en/_index.json");

fn en() -> bool { vc_i18n::language() == vc_i18n::Lang::En }

fn en_index() -> &'static Value {
    static V: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    V.get_or_init(|| serde_json::from_str(EN_INDEX).expect("help/en/_index.json"))
}

impl Topic {
    /// Text der Seite in der aktuellen Sprache (Englisch aus `help/en`, sonst und bei fehlender Übersetzung Deutsch)
    pub fn body_l(&self) -> &'static str {
        if en() { if let Some((_, b)) = EN_BODIES.iter().find(|(i, _)| *i == self.id) { return b; } }
        self.body
    }
    pub fn title_l(&self) -> String {
        if en() { if let Some(t) = en_index()["titles"][self.id].as_str().filter(|t| !t.is_empty()) { return t.to_string(); } }
        self.title.to_string()
    }
    pub fn group_l(&self) -> String {
        if en() { if let Some(t) = en_index()["groups"][self.group].as_str().filter(|t| !t.is_empty()) { return t.to_string(); } }
        self.group.to_string()
    }
}

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
        "pet" => &["SYS-PET"],
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
    let hb = handbook().iter().filter(|t| visible_with(t.id, &f)).map(|t| json!({ "id": t.id, "group": t.group_l(), "title": t.title_l() }));
    json!({ "topics": hb.chain(TOPICS.iter().filter(|t| visible_with(t.id, &f)).map(|t| json!({ "id": t.id, "group": t.group_l(), "title": t.title_l() }))).collect::<Vec<_>>() })
}

/// Kurzname eines Systems für die Seite „Emulatoren“
fn system_short(id: &str) -> String {
    match id { "c64" => "C64", "c128" => "C128", "plus4" => "Plus/4", "c16" => "C16/C116", "vic20" => if en() { "VIC-20" } else { "VC-20" }, "atari8" => "Atari 800 XL/XE", "mega65" => "MEGA65", other => other }.to_string()
}

fn mb(b: u64) -> String { format!("{:.1} MB", b as f64 / 1_048_576.0) }

/// Zeile je Betriebssystem eines Emulators: ladbare Pakete oder der Hinweis aus dem Katalog
fn os_lines(e: &crate::emulators::Entry) -> Vec<(String, String)> {
    ["windows", "linux", "macos"].iter().filter_map(|os| {
        let pk: Vec<String> = e.packages.iter().filter(|p| p.os == *os && p.sha256.len() == 64).map(|p| format!("{} {} ({})", p.version, if p.arch == "aarch64" { "ARM".to_string() } else { vc_i18n::t!("help-emu-arch-64bit") }, mb(p.size))).collect();
        let text = if !pk.is_empty() { vc_i18n::t!("help-emu-portable-download", pakete = pk.join(", ")) } else { e.manual.get(*os).cloned().filter(|h| !h.is_empty())? };
        Some((match *os { "windows" => "Windows", "linux" => "Linux", _ => "macOS" }.to_string(), text))
    }).collect()
}

/// Tabelle „Übersicht“ der Seite „Emulatoren“ aus dem Katalog (nur gewählte Anbindungen)
fn emulators_table(es: &[&crate::emulators::Entry]) -> String {
    let mut t = format!("| {} | {} | {} | {} |\n|---|---|---|---|\n", vc_i18n::t!("help-emu-col-emulator"), vc_i18n::t!("help-emu-col-systems"), vc_i18n::t!("help-emu-col-role"), vc_i18n::t!("help-emu-col-portable"));
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
        out.push_str(&format!("## {}\n\n{}\n\n- **{}:** [{h}]({h})\n- **{}:** [{d}]({d})\n- **{}:** {}\n- **{}:** {}\n- **{}:** {}\n", e.name, e.summary, vc_i18n::t!("help-emu-homepage"), vc_i18n::t!("help-emu-download-page"), vc_i18n::t!("help-emu-license"), e.license, vc_i18n::t!("help-emu-col-systems"), e.systems.iter().map(|s| system_short(s)).collect::<Vec<_>>().join(", "), vc_i18n::t!("help-emu-role-in-vc"), e.role, h = e.homepage, d = e.download_page));
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
    handbook().iter().chain(TOPICS.iter()).find(|t| t.id == id && visible(t.id)).map(|t| json!({ "kind": "topic", "id": t.id, "group": t.group_l(), "title": t.title_l(), "markdown": if t.id == "emulators" { emulators_page(t.body_l()) } else { t.body_l().to_string() } }))
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
    include_str!("../../../docs/handbuch/kapitel/21-lizenz.md"),
];

/// Englische Kapitel (`docs/handbuch/en/kapitel`, gleiche Dateinamen)
const CHAPTERS_EN: &[&str] = &[
    include_str!("../../../docs/handbuch/en/kapitel/00-installation.md"),
    include_str!("../../../docs/handbuch/en/kapitel/01-willkommen.md"),
    include_str!("../../../docs/handbuch/en/kapitel/02-erstes-projekt.md"),
    include_str!("../../../docs/handbuch/en/kapitel/03-oberflaeche.md"),
    include_str!("../../../docs/handbuch/en/kapitel/04-projekte.md"),
    include_str!("../../../docs/handbuch/en/kapitel/05-editor.md"),
    include_str!("../../../docs/handbuch/en/kapitel/06-assembler.md"),
    include_str!("../../../docs/handbuch/en/kapitel/07-basic.md"),
    include_str!("../../../docs/handbuch/en/kapitel/08-bauen-starten.md"),
    include_str!("../../../docs/handbuch/en/kapitel/09-debugger.md"),
    include_str!("../../../docs/handbuch/en/kapitel/10-tests.md"),
    include_str!("../../../docs/handbuch/en/kapitel/11-grafik-ueberblick.md"),
    include_str!("../../../docs/handbuch/en/kapitel/12-zeichensatz.md"),
    include_str!("../../../docs/handbuch/en/kapitel/13-sprites.md"),
    include_str!("../../../docs/handbuch/en/kapitel/14-bildschirm-kacheln.md"),
    include_str!("../../../docs/handbuch/en/kapitel/15-paletten.md"),
    include_str!("../../../docs/handbuch/en/kapitel/16-tracker.md"),
    include_str!("../../../docs/handbuch/en/kapitel/17-play-tracker.md"),
    include_str!("../../../docs/handbuch/en/kapitel/18-disk-studio.md"),
    include_str!("../../../docs/handbuch/en/kapitel/19-systeme.md"),
    include_str!("../../../docs/handbuch/en/kapitel/20-kuerzel-faq.md"),
    include_str!("../../../docs/handbuch/en/kapitel/21-lizenz.md"),
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
        v["bilder"].as_array().map(|a| a.iter().filter(|b| b["status"].as_str() == Some("vorhanden")).filter_map(|b| Some((b["id"].as_str()?.to_string(), b["alt"].as_str().unwrap_or("").to_string(), b["caption"].as_str().unwrap_or("").to_string()))).collect()).unwrap_or_default()
    })
}

/// Gibt es das Bild `id` (Datei `<id>.png`)? Unbekannte Kennungen lässt die Hilfe weg.
pub fn handbook_image_known(id: &str) -> bool { handbook_images().iter().any(|(i, _, _)| i == id) }

/// Handbuchkapitel als Hilfethemen, nach `reihenfolge` sortiert; Kennung `kapitel:<id>`, Gruppe „Handbuch“
pub fn handbook() -> &'static [Topic] {
    static H: [std::sync::OnceLock<Vec<Topic>>; 2] = [std::sync::OnceLock::new(), std::sync::OnceLock::new()];
    H[usize::from(en())].get_or_init(|| {
        let mut ch: Vec<(ChapterMeta, &'static str)> = (if en() { CHAPTERS_EN } else { CHAPTERS }).iter().filter_map(|c| parse_frontmatter(c)).collect();
        ch.sort_by_key(|(m, _)| m.order);
        let title_of = |id: &str| ch.iter().find(|(m, _)| m.id == id).map(|(m, _)| m.title.clone());
        ch.iter().map(|(m, body)| {
            let (first, rest) = body.split_once('\n').unwrap_or((body, ""));
            let mut md = format!("{first}\n\n*{}*\n{rest}", vc_i18n::t!("help-chapter-meta", stufe = m.level.as_str(), zeit = m.time.as_str()));
            let see: Vec<String> = m.see_also.iter().filter_map(|i| title_of(i).map(|t| format!("[{t}](kapitel:{i})"))).collect();
            if !see.is_empty() { md.push_str(&format!("\n\n**{}** {}", vc_i18n::t!("help-chapter-see-also"), see.join(" · "))); }
            let refs: Vec<String> = m.reference.iter().filter_map(|i| TOPICS.iter().find(|t| t.id == i).map(|t| format!("[{}](hilfe:{i})", t.title_l()))).collect();
            if !refs.is_empty() { md.push_str(&format!("\n\n**{}** {}", vc_i18n::t!("help-chapter-reference"), refs.join(" · "))); }
            md.push('\n');
            Topic { id: Box::leak(format!("kapitel:{}", m.id).into_boxed_str()), group: if en() { Box::leak(en_index()["groups"][HANDBOOK_GROUP].as_str().filter(|g| !g.is_empty()).unwrap_or(HANDBOOK_GROUP).to_string().into_boxed_str()) } else { HANDBOOK_GROUP }, title: Box::leak(m.title.clone().into_boxed_str()), body: Box::leak(md.into_boxed_str()) }
        }).collect()
    })
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
    if quad { md.push_str(&format!("{}\n\n", vc_i18n::t!("help-mn-quad", base = base.as_str()))); }
    if ops.iter().any(|o| o.mode == Mode::Izz) && !quad { md.push_str(&format!("{}\n\n", vc_i18n::t!("help-mn-izz"))); }
    md.push_str(&format!("{}\n|---|---|---|---|---|---|\n", vc_i18n::t!("help-mn-table-head")));
    let mut any_illegal = false;
    for o in &ops {
        let syn = o.mode.syntax(); any_illegal |= o.group != "official";
        let pre = if quad { 2 } else { 0 };
        md.push_str(&format!("| {} | `{} {}` | `{}${:02X}` | {} | {}{}{} | {} |\n", o.mode.label_l(), w.to_ascii_lowercase(), syn.to_ascii_lowercase(), if quad { "42 42 " } else { "" }, o.opcode, o.bytes + pre, o.cycles,
            if o.page_cross { "+1" } else { "" }, if o.branch { "+1/+2" } else { "" }, if o.affects.is_empty() { "–".into() } else { o.affects.clone() }));
    }
    if any_illegal { md.push_str(&format!("\n{}\n", vc_i18n::t!("help-mn-illegal"))); }
    md.push_str(&format!("\n{}\n", vc_i18n::t!("help-mn-cycles-flags")));
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
        let mut v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        vc_i18n::localize_json(&mut v, "help/basic_keywords.json");
        let key = w.to_ascii_uppercase();
        if let Some(k) = v["keywords"].get(&key).or_else(|| v["keywords"].get(format!("{key}$"))) {
            let md = format!("# {key}\n\n{}\n\n```\n{}\n```\n\n{}", k["text"].as_str().unwrap_or(""), k["example"].as_str().unwrap_or(""), vc_i18n::t!("help-see-basic"));
            return Ok(json!({ "kind": "keyword", "id": key, "title": key, "markdown": md, "topics": ["basic"] }));
        }
    }
    if w.starts_with('.') {
        if let Some(d) = vc_lsp::directive_doc(&w) {
            let rel = if matches!(w.as_str(), ".bank" | ".segment" | ".cartridge") { "cartridges" } else { "assembler" };
            return Ok(json!({ "kind": "directive", "id": w, "title": w, "markdown": format!("# {w}\n\n{d}\n\n{}", vc_i18n::t!("help-see-topic", title = TOPICS.iter().find(|t| t.id == rel).map(|t| t.title_l()).unwrap_or_default(), rel = rel)), "topics": [rel] }));
        }
    }
    if let Ok((t, cpu)) = target_cpu(data, target) {
        if let Some(p) = mnemonic_page(&cpu, &w) { return Ok(p); }
        let regs = register_pages(data, &t, &w);
        if !regs.is_empty() {
            let mut md = format!("# {}\n\n", if w.contains('.') { w.clone() } else { vc_i18n::t!("help-reg-title", name = w.as_str()) });
            for (c, n, a, acc, d) in &regs { md.push_str(&format!("{}\n\n{d}\n\n", vc_i18n::t!("help-reg-line", chip = c.as_str(), name = n.as_str(), addr = format!("${a:04X}"), access = acc.as_str()))); }
            md.push_str(&vc_i18n::t!("help-reg-use"));
            return Ok(json!({ "kind": "register", "id": w, "title": w, "markdown": md, "topics": ["assembler"] }));
        }
    }
    let syms = crate::sysmap::symbols(data, target);
    if let Some(sym) = crate::sysmap::find(&syms, &w) {
        return Ok(json!({ "kind": "symbol", "id": sym.name, "title": sym.name, "markdown": crate::sysmap::page(&syms, sym), "topics": ["assembler"] }));
    }
    if let Some(t) = TOPICS.iter().find(|t| (t.id == w || t.title.to_lowercase() == w || t.title_l().to_lowercase() == w) && visible(t.id)) { return Ok(topic(t.id).unwrap()); }
    Ok(json!({ "kind": "none", "title": w, "markdown": vc_i18n::t!("help-none", word = raw) }))
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
        let (title, body) = (t.title_l().to_lowercase(), t.body_l().to_lowercase());
        let n = body.matches(&q).count() as i32;
        let score = if title.contains(&q) { 50 } else { 0 } + n.min(10) * 2 + if t.id == q { 60 } else { 0 };
        if score > 0 { res.push((score, json!({ "kind": "topic", "id": t.id, "title": t.title_l(), "snippet": snippet(t.body_l(), &q) }))); }
    }
    for (d, doc) in vc_lsp::DIRECTIVES { if d.contains(&q) || (q.len() > 2 && doc.to_lowercase().contains(&q)) { res.push((if d.trim_start_matches('.') == q.trim_start_matches('.') { 70 } else { 20 }, json!({ "kind": "directive", "id": d, "title": d, "snippet": doc }))); } }
    if let Ok((t, cpu)) = target_cpu(data, target) {
        let mut names: Vec<String> = cpu.ops.iter().map(|o| o.mnemonic.to_lowercase()).chain(cpu.quad_ops.iter().map(|(q, _)| q.to_lowercase())).collect(); names.sort(); names.dedup();
        for n in names.iter().filter(|n| n.starts_with(&q)).take(8) { res.push((if *n == q { 80 } else { 15 }, json!({ "kind": "mnemonic", "id": n, "title": n.to_uppercase(), "snippet": vc_lsp::mnemonic_doc(&n.to_uppercase()).unwrap_or("") }))); }
        let mut seen = 0;
        'outer: for c in &t.chips { if let Ok(chip) = load_chip(data, c) { for r in &chip.regs { if r.name.contains(&q) || format!("{}.{}", chip.id, r.name).contains(&q) { res.push((if r.name == q { 40 } else { 10 }, json!({ "kind": "register", "id": format!("{}.{}", chip.id, r.name), "title": format!("{}.{}", chip.id, r.name), "snippet": format!("${:04X} {}", r.addr, r.doc) }))); seen += 1; if seen >= 10 { break 'outer; } } } } }
    }
    if let Ok(text) = vc_core::read_static(&data.join("help/basic_keywords.json")) {
        if let Ok(mut v) = serde_json::from_str::<Value>(&text) {
            vc_i18n::localize_json(&mut v, "help/basic_keywords.json");
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

    /// Englisch: jede Seite und jeder Titel ist übersetzt, Linkziele stimmen mit dem Deutschen überein, nichts Deutsches bleibt stehen
    #[test]
    fn englische_hilfe_ist_vollstaendig() {
        let umlaut = |t: &str| t.chars().any(|c| "äöüÄÖÜß".contains(c));
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::En));
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("help");
        let veraltet_de: Vec<String> = en_index()["files"].as_object().into_iter().flatten()
            .filter(|(_, v)| v["status"].as_str() == Some("veraltet"))
            .filter_map(|(k, _)| std::fs::read_to_string(dir.join(k)).ok()).collect();
        for t in TOPICS {
            let (de, en) = (t.body, t.body_l());
            assert!(de != en, "{}: keine englische Seite", t.id);
            assert!(en.starts_with("# "), "{}", t.id);
            assert!(!umlaut(en), "{}: deutsche Zeichen auf der englischen Seite", t.id);
            assert!(en_index()["titles"][t.id].as_str().is_some_and(|x| !x.is_empty()), "{}: Titel nicht übersetzt", t.id);
            assert!(!umlaut(&t.title_l()) && !umlaut(&t.group_l()), "{}", t.id);
            let links = |b: &str| { let mut v: Vec<String> = b.split("](").skip(1).map(|x| x.split(')').next().unwrap_or("").split(' ').next().unwrap_or("").to_string()).collect(); v.sort(); v };
            // Seiten mit Status „veraltet“ (deutsch neu geschrieben, englisch folgt in 0.3.1) sind von der Strukturprüfung ausgenommen
            let veraltet = veraltet_de.iter().any(|d| d == de);
            if !veraltet { assert_eq!(links(de), links(en), "{}: Linkziele weichen ab", t.id); }
        }
        let list = topic_list().to_string();
        assert!(list.contains("Manual"), "Gruppe des Handbuchs: {list}");
        let md = topic("emulators").unwrap()["markdown"].as_str().unwrap().to_string();
        assert!(md.contains("Homepage") && md.contains("| Emulator | Systems |"), "{md}");
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        assert_eq!(TOPICS[0].title_l(), TOPICS[0].title);
    }

    /// Englisches Handbuch: gleiche Kapitel wie das deutsche, Titel und Text übersetzt, Verweise und Bilder gültig
    #[test]
    fn englisches_handbuch_ist_vollstaendig() {
        let umlaut = |t: &str| t.chars().any(|c| "äöüÄÖÜß".contains(c));
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        let de: Vec<(String, String)> = handbook().iter().map(|t| (t.id.to_string(), t.title.to_string())).collect();
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::En));
        let en = handbook();
        assert_eq!(en.len(), de.len(), "Anzahl der Kapitel");
        assert_eq!(en.len(), 22);
        for (t, (id, title)) in en.iter().zip(&de) {
            assert_eq!(t.id, id);
            assert!(t.title != title || title.chars().all(|c| c.is_ascii()), "{id}: Titel nicht übersetzt");
            assert!(!umlaut(t.title) && !umlaut(t.body), "{id}: deutsche Zeichen im englischen Kapitel");
            assert_eq!(t.group, "Manual");
            for img in links_of(t.body, "bild") { assert!(handbook_image_known(&img), "{id}: Bild {img} unbekannt"); }
            for k in links_of(t.body, "kapitel") { assert!(topic(&format!("kapitel:{k}")).is_some(), "{id}: Kapitel {k} unbekannt"); }
            for h in links_of(t.body, "hilfe").into_iter().chain(links_of(t.body, "help")) { assert!(TOPICS.iter().any(|x| x.id == h), "{id}: Hilfethema {h} unbekannt"); }
        }
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        assert_eq!(handbook()[0].group, HANDBOOK_GROUP);
    }
}

// ---- Themenregister mit Quellen: eingebaute Themen und Hilfeseiten aus Erweiterungen (`help/index.json` + `help/*.md` in den Quellen der Registry) ----

/// Ein Hilfethema aus einer Erweiterung; die Kennung heißt `<paket>:<id>` (kann nie mit einem eingebauten Thema kollidieren)
#[derive(Clone, Debug, PartialEq)]
pub struct ExternesThema { pub id: String, pub group: String, pub title: String, pub body: String, pub herkunft: String }

/// Hilfethemen aus den Quellen außer dem Programm. `index.json` ist eine Liste von `{ "id", "group", "title", "file" }`, `file` liegt in `help/`.
/// Fehlerhafte Einträge (kein Name, Datei fehlt, Pfad nach außen, zu groß) werden übersprungen und gemeldet.
pub fn externe_themen(r: &crate::registry::Registry) -> (Vec<ExternesThema>, Vec<crate::registry::Problem>) {
    use crate::registry::{Problem, Quelle};
    const MAX: u64 = 1024 * 1024;
    let (mut themen, mut probleme) = (vec![], vec![]);
    for (w, dir) in r.suchpfade("help") {
        if w.quelle == Quelle::Programm { continue; }
        let index = dir.join("index.json"); let paket = w.herkunft();
        let problem = |text: String| Problem { quelle: w.quelle, herkunft: paket.clone(), datei: index.clone(), text };
        let v: Value = match std::fs::read_to_string(&index).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) { Ok(v) => v, Err(e) => { probleme.push(problem(e)); continue; } };
        for (i, e) in v.as_array().into_iter().flatten().enumerate() {
            let (id, titel, file) = (e["id"].as_str().unwrap_or(""), e["title"].as_str().unwrap_or(""), e["file"].as_str().unwrap_or(""));
            if id.is_empty() || titel.is_empty() || file.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') { probleme.push(problem(format!("Thema {} unvollständig (id, title, file)", i + 1))); continue; }
            if file.split(['/', '\\']).any(|t| t == ".." || t.is_empty()) || file.starts_with('/') || !file.ends_with(".md") { probleme.push(problem(format!("{id}: Dateiname „{file}“ ist nicht erlaubt"))); continue; }
            let p = dir.join(file);
            match std::fs::metadata(&p) { Ok(m) if m.is_file() && m.len() <= MAX => {}, Ok(m) if m.len() > MAX => { probleme.push(problem(format!("{id}: Datei zu groß"))); continue; }, _ => { probleme.push(problem(format!("{id}: Datei „{file}“ fehlt"))); continue; } }
            let Ok(body) = std::fs::read_to_string(&p) else { probleme.push(problem(format!("{id}: Datei „{file}“ nicht lesbar"))); continue; };
            themen.push(ExternesThema { id: format!("{paket}:{id}"), group: e["group"].as_str().filter(|g| !g.is_empty()).unwrap_or(&paket).to_string(), title: titel.to_string(), body, herkunft: paket.clone() });
        }
    }
    (themen, probleme)
}

/// Themenliste wie [`topic_list`], ergänzt um die Themen aus Erweiterungen (mit `source`)
pub fn topic_list_mit(r: &crate::registry::Registry) -> Value {
    let mut v = topic_list();
    if let Some(a) = v["topics"].as_array_mut() { for t in externe_themen(r).0 { a.push(json!({ "id": t.id, "group": t.group, "title": t.title, "source": t.herkunft })); } }
    v
}

/// Thema nach Kennung, auch aus Erweiterungen (`<paket>:<id>`)
pub fn topic_mit(r: &crate::registry::Registry, id: &str) -> Option<Value> {
    let id = id.strip_prefix("hilfe:").or_else(|| id.strip_prefix("help:")).unwrap_or(id);
    externe_themen(r).0.into_iter().find(|t| t.id == id).map(|t| json!({ "kind": "topic", "id": t.id, "group": t.group, "title": t.title, "markdown": t.body, "source": t.herkunft })).or_else(|| topic(id))
}

#[cfg(test)]
mod themenregister_tests {
    use super::*;
    use crate::registry::{Quelle, Registry};
    #[test]
    fn themen_aus_erweiterungen_mit_namensraum_und_meldungen() {
        let t = std::env::temp_dir().join(format!("vc_hilfe_{}", std::process::id())); let _ = std::fs::remove_dir_all(&t);
        let h = t.join("ext/autor.demo/help"); std::fs::create_dir_all(&h).unwrap();
        std::fs::write(h.join("index.json"), r#"[{"id":"erste-schritte","group":"Demo","title":"Erste Schritte","file":"erste.md"},{"id":"fehlt","title":"X","file":"nein.md"},{"id":"boese","title":"B","file":"../x.md"},{"id":"Gross","title":"G","file":"g.md"},{"title":"ohne id","file":"a.md"}]"#).unwrap();
        std::fs::write(h.join("erste.md"), "# Hallo\n\nText.").unwrap();
        let r = Registry::programm(None).mit_paketordner(Quelle::Benutzer, &t.join("ext"));
        let (th, pr) = externe_themen(&r);
        assert_eq!(th.len(), 1); assert_eq!((th[0].id.as_str(), th[0].group.as_str(), th[0].herkunft.as_str()), ("autor.demo:erste-schritte", "Demo", "autor.demo"));
        assert_eq!(pr.len(), 4, "{pr:?}");
        let l = topic_list_mit(&r); assert!(l["topics"].as_array().unwrap().iter().any(|x| x["id"] == "autor.demo:erste-schritte" && x["source"] == "autor.demo"));
        assert!(l["topics"].as_array().unwrap().iter().any(|x| x["id"] == "start"), "eingebaute Themen bleiben");
        assert_eq!(topic_mit(&r, "help:autor.demo:erste-schritte").unwrap()["markdown"], "# Hallo\n\nText.");
        assert!(topic_mit(&r, "start").is_some() && topic_mit(&r, "autor.demo:nein").is_none());
        std::fs::remove_dir_all(t).ok();
    }
}
