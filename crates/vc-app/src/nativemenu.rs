// SPDX-License-Identifier: GPL-3.0-or-later
//! Menüleiste am oberen Bildschirmrand (macOS, native Menüs über `muda`). Auf anderen Systemen bleibt die Menüleiste im Fenster
//! (`app.rs`); dieses Modul ist dort leer.
//!
//! Die Einträge lösen dieselben Aktionen aus wie die Menüs im Fenster: `Native::poll` liefert die angeklickten Befehle,
//! `Native::sync` setzt Aktivierung und Haken.

/// Menübefehle (gleich auf allen Systemen, damit `app.rs` ohne `cfg` darauf antworten kann)
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]   // nur das macOS-Menü erzeugt sie
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cmd {
    NewProject, OpenProject, Settings, NewFile, NewMedia, ConvertBasic, StdCharset, ImportRom, ImportSheet, Migrate, Save, RomSetup, RomFolder, RomCheck, SetupAll, SetupRoms, SetupEmulators, SetupHardware, SetupDesign, SetupExport, SetupImport, SetupReset,
    Build, Run, Debug, BuildActive, RunActive, DebugActive, ProjectSettings,
    InsertChar, Palette, LiveImage,
    About, Feedback, StudioDetach, StudioEmbed, HotKeys, ShowAddr, SchemeDark, SchemeLight, SchemeSystem, ZoomIn, ZoomOut, ZoomReset, FontBigger, FontSmaller, ResetView,
    SaveAll, CloseFile, SplitEditor, ToggleSide, ToggleInspector, ToggleFooter, WorkCode, WorkDebug, Maximize, ResetLayout,
    DbgCont, DbgPause, DbgStepInto, DbgStepOver, DbgStepOut, DbgStop, DbgMemory, DbgEmulator, Help, Shortcuts,
    Undo, Redo, Find, Replace, GotoLine, FindInProject, Export,
    /// Eintrag der Liste „Zuletzt geöffnet“ (Nummer in der Liste)
    Recent(u8),
}

/// Zustand, von dem Aktivierung und Haken abhängen
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuState {
    pub project: bool, pub idle: bool, pub can_debug: bool, pub convert: bool, pub legacy: bool,
    pub detached: bool, pub hotkeys: bool, pub show_addr: bool, pub scheme: u8,
    /// Sichtbarkeit der Zonen des aktuellen Arbeitsplatzes, Arbeitsplatz (true = Debug), zeitweise ausgeblendete Zonen, geteilter Editor, offene Datei
    pub side: bool, pub inspector: bool, pub footer: bool, pub debug_wp: bool, pub zen: bool, pub split: bool, pub has_doc: bool, pub studio: bool,
    /// Debug-Sitzung vorhanden, angehalten (Schritte möglich)
    pub debugging: bool, pub halted: bool,
}

/// Ein Befehl mit Beschriftung, Gruppe und Tastenkürzel. Menüleiste und Befehlspalette lesen beide diese Tabelle.
/// `keys`: Kürzel in der Schreibweise „Cmd+Umschalt+Y“ bzw. „F7“ (`Cmd` heißt unter Windows und Linux Strg, siehe [`keys_text`]).
pub struct CmdInfo { pub cmd: Cmd, pub group: &'static str, pub label: &'static str, pub keys: &'static str }

const fn ci(cmd: Cmd, group: &'static str, label: &'static str, keys: &'static str) -> CmdInfo { CmdInfo { cmd, group, label, keys } }

pub const COMMANDS: &[CmdInfo] = &[
    ci(Cmd::Palette, "Ansicht", "Befehl oder Datei suchen …", "Cmd+K"),
    ci(Cmd::About, "Hilfe", "Über VintageCoder", ""),
    ci(Cmd::Feedback, "Hilfe", "Feedback senden …", ""),
    ci(Cmd::Settings, "VintageCoder", "Einstellungen …", "Cmd+,"),
    ci(Cmd::NewProject, "Datei", "Neues Projekt …", "Cmd+Umschalt+N"),
    ci(Cmd::OpenProject, "Datei", "Projekt öffnen …", "Cmd+O"),
    ci(Cmd::NewFile, "Datei", "Neue Quelldatei …", "Cmd+N"),
    ci(Cmd::NewMedia, "Datei", "Neu: Sprite, Zeichensatz, Song …", ""),
    ci(Cmd::ConvertBasic, "Datei", "BASIC nach Assembler konvertieren …", ""),
    ci(Cmd::StdCharset, "Datei", "Standardzeichensatz hinzufügen …", ""),
    ci(Cmd::ImportRom, "Datei", "Zeichensatz aus ROM importieren …", ""),
    ci(Cmd::ImportSheet, "Datei", "Zeichensatz aus PNG-Raster importieren …", ""),
    ci(Cmd::Migrate, "Datei", "Projekt auf neue Dateinamen umstellen …", ""),
    ci(Cmd::Save, "Datei", "Speichern", "Cmd+S"),
    ci(Cmd::SaveAll, "Datei", "Alle speichern", "Cmd+Alt+S"),
    ci(Cmd::CloseFile, "Datei", "Datei schließen", "Cmd+W"),
    ci(Cmd::RomSetup, "Datei", "ROMs einrichten …", ""),
    ci(Cmd::RomFolder, "Datei", "ROM-Ordner öffnen", ""),
    ci(Cmd::RomCheck, "Datei", "ROMs prüfen", ""),
    ci(Cmd::SetupAll, "Hilfe", "Konfigurationsassistent: alles einrichten …", ""),
    ci(Cmd::SetupRoms, "Hilfe", "Konfigurationsassistent: nur ROMs …", ""),
    ci(Cmd::SetupEmulators, "Hilfe", "Konfigurationsassistent: nur Emulatoren …", ""),
    ci(Cmd::SetupHardware, "Hilfe", "Konfigurationsassistent: nur echte Hardware …", ""),
    ci(Cmd::SetupDesign, "Hilfe", "Konfigurationsassistent: nur Design und Editor …", ""),
    ci(Cmd::SetupExport, "Hilfe", "Konfiguration exportieren …", ""),
    ci(Cmd::SetupImport, "Hilfe", "Konfiguration importieren …", ""),
    ci(Cmd::SetupReset, "Hilfe", "Konfiguration auf Standard zurücksetzen …", ""),
    ci(Cmd::Build, "Projekt", "Projekt bauen", "F7"),
    ci(Cmd::Run, "Projekt", "Startprogramm starten", "F5"),
    ci(Cmd::Debug, "Projekt", "Startprogramm debuggen", "F6"),
    ci(Cmd::BuildActive, "Projekt", "Aktive Datei bauen", "Umschalt+F7"),
    ci(Cmd::RunActive, "Projekt", "Aktive Datei starten", "Umschalt+F5"),
    ci(Cmd::DebugActive, "Projekt", "Aktive Datei debuggen", "Umschalt+F6"),
    ci(Cmd::ProjectSettings, "Projekt", "Projekt-Einstellungen …", ""),
    ci(Cmd::LiveImage, "Projekt", "Live-Bild öffnen … (Ultimate 64)", ""),
    ci(Cmd::InsertChar, "Bearbeiten", "Zeichen einfügen … (PETSCII/ATASCII)", "Cmd+Umschalt+Y"),
    ci(Cmd::SchemeDark, "Farbschema", "Dunkel", ""),
    ci(Cmd::SchemeLight, "Farbschema", "Hell", ""),
    ci(Cmd::SchemeSystem, "Farbschema", "Wie das System", ""),
    ci(Cmd::ZoomIn, "Ansicht", "Oberfläche vergrößern", "Cmd++"),
    ci(Cmd::ZoomOut, "Ansicht", "Oberfläche verkleinern", "Cmd+-"),
    ci(Cmd::ZoomReset, "Ansicht", "Oberfläche: Originalgröße", "Cmd+0"),
    ci(Cmd::FontBigger, "Ansicht", "Editor-Schrift größer", ""),
    ci(Cmd::FontSmaller, "Ansicht", "Editor-Schrift kleiner", ""),
    ci(Cmd::ResetView, "Ansicht", "Darstellung zurücksetzen", ""),
    ci(Cmd::ShowAddr, "Ansicht", "Adressen im Editor (BASIC: Zeilennummern)", ""),
    ci(Cmd::HotKeys, "Ansicht", "Einzeltasten im Grafik-Editor", ""),
    ci(Cmd::Export, "Datei", "Exportieren …", ""),
    ci(Cmd::Undo, "Bearbeiten", "Rückgängig", "Cmd+Z"),
    ci(Cmd::Redo, "Bearbeiten", "Wiederholen", "Cmd+Umschalt+Z"),
    ci(Cmd::Find, "Bearbeiten", "Suchen …", "Cmd+F"),
    ci(Cmd::Replace, "Bearbeiten", "Ersetzen …", "Cmd+Alt+F"),
    ci(Cmd::GotoLine, "Bearbeiten", "Gehe zu Zeile …", "Cmd+L"),
    ci(Cmd::FindInProject, "Bearbeiten", "In Projekt suchen …", "Cmd+Umschalt+F"),
    ci(Cmd::ToggleSide, "Ansicht", "Seitenleiste", "Cmd+B"),
    ci(Cmd::ToggleInspector, "Ansicht", "Inspektor", "Cmd+Alt+B"),
    ci(Cmd::ToggleFooter, "Ansicht", "Fußpanel", "Cmd+J"),
    ci(Cmd::WorkCode, "Ansicht", "Arbeitsplatz Code", ""),
    ci(Cmd::WorkDebug, "Ansicht", "Arbeitsplatz Debug", ""),
    ci(Cmd::SplitEditor, "Ansicht", "Editor teilen / Teilung aufheben", "Cmd+\\"),
    ci(Cmd::Maximize, "Ansicht", "Maximieren (Seitenleiste, Inspektor, Fußpanel aus)", "Cmd+Umschalt+Enter"),
    ci(Cmd::ResetLayout, "Ansicht", "Layout zurücksetzen", ""),
    ci(Cmd::DbgCont, "Debug", "Weiter", "F8"),
    ci(Cmd::DbgPause, "Debug", "Pause", ""),
    ci(Cmd::DbgStepInto, "Debug", "Schritt", "F11"),
    ci(Cmd::DbgStepOver, "Debug", "Über", "F10"),
    ci(Cmd::DbgStepOut, "Debug", "Raus (bis Rücksprung)", "Umschalt+F11"),
    ci(Cmd::DbgStop, "Debug", "Debug-Sitzung beenden", ""),
    ci(Cmd::DbgMemory, "Debug", "Speicher anzeigen", ""),
    ci(Cmd::DbgEmulator, "Debug", "Emulator-Bild anzeigen", ""),
    ci(Cmd::Help, "Hilfe", "Hilfe", "F1"),
    ci(Cmd::Shortcuts, "Hilfe", "Tastenkürzel", ""),
    ci(Cmd::StudioDetach, "Fenster", "Studio in eigenem Fenster lösen", ""),
    ci(Cmd::StudioEmbed, "Fenster", "Gelöste Studios zurückholen", ""),
];

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]   // nur das macOS-Menü fragt einzelne Befehle ab
pub fn info(c: Cmd) -> &'static CmdInfo { COMMANDS.iter().find(|i| i.cmd == c).expect("Befehl fehlt in COMMANDS") }

/// Kürzel zur Anzeige: auf dem Mac `⌘`/`⇧`, sonst `Strg`/`Umschalt`
pub fn keys_text(keys: &str, mac: bool) -> String {
    if keys.is_empty() { return String::new(); }
    if mac { keys.replace("Cmd+Umschalt+", "⇧⌘").replace("Cmd+", "⌘").replace("Umschalt+", "⇧") }
    else { keys.replace("Cmd", "Strg") }
}

/// Ist der Befehl im Zustand `s` verfügbar? (Menü: aktiviert; Befehlspalette: wählbar)
pub fn enabled(c: Cmd, s: &MenuState) -> bool {
    use Cmd::*;
    match c {
        NewFile | NewMedia | StdCharset | ImportRom | ImportSheet | Save | SaveAll | ProjectSettings | InsertChar | FindInProject => s.project,
        CloseFile => s.has_doc,
        Undo | Redo | Find | Replace | GotoLine => s.has_doc || s.studio,
        Export | StudioDetach => s.studio,
        StudioEmbed => s.detached,
        DbgPause | DbgStop => s.debugging,
        DbgCont | DbgStepInto | DbgStepOver | DbgStepOut => s.debugging && s.halted,
        ConvertBasic => s.project && s.convert,
        Migrate => s.legacy,
        Build | Run | BuildActive | RunActive => s.idle,
        Debug | DebugActive => s.idle && s.can_debug,
        _ => true,
    }
}

/// Eintrag einer Menüleiste
#[derive(Clone, Copy)]
pub enum M { Item(Cmd), Check(Cmd), Sep, Sub(&'static str, &'static [M]), /// Untermenü „Zuletzt geöffnet“ (Einträge kommen zur Laufzeit)
    Recent }

/// Aufbau der Menüleiste (macOS: am Bildschirmrand, sonst in der Titelzeile) nach dem Layoutkonzept. Beide lesen diese Tabelle.
/// `Settings` und `About` stehen auf dem Mac im Programmmenü und werden dort aus dieser Tabelle ausgelassen.
pub const MENUS: &[(&str, &[M])] = &[
    ("Datei", &[M::Item(Cmd::NewProject), M::Item(Cmd::OpenProject), M::Recent, M::Sep, M::Item(Cmd::NewFile), M::Item(Cmd::NewMedia), M::Sep,
        M::Item(Cmd::Save), M::Item(Cmd::SaveAll), M::Item(Cmd::CloseFile), M::Sep, M::Item(Cmd::Export), M::Sep,
        M::Item(Cmd::ConvertBasic), M::Item(Cmd::StdCharset), M::Item(Cmd::ImportRom), M::Item(Cmd::ImportSheet), M::Item(Cmd::Migrate), M::Sep, M::Item(Cmd::Settings)]),
    ("Bearbeiten", &[M::Item(Cmd::Undo), M::Item(Cmd::Redo), M::Sep, M::Item(Cmd::Find), M::Item(Cmd::Replace), M::Item(Cmd::GotoLine), M::Item(Cmd::FindInProject), M::Sep, M::Item(Cmd::InsertChar)]),
    ("Ansicht", &[M::Item(Cmd::Palette), M::Sep,
        M::Check(Cmd::ToggleSide), M::Check(Cmd::ToggleInspector), M::Check(Cmd::ToggleFooter), M::Sep,
        M::Check(Cmd::WorkCode), M::Check(Cmd::WorkDebug), M::Item(Cmd::SplitEditor), M::Item(Cmd::Maximize), M::Item(Cmd::ResetLayout), M::Sep,
        M::Sub("Farbschema", &[M::Check(Cmd::SchemeDark), M::Check(Cmd::SchemeLight), M::Check(Cmd::SchemeSystem)]), M::Sep,
        M::Item(Cmd::ZoomIn), M::Item(Cmd::ZoomOut), M::Item(Cmd::ZoomReset), M::Item(Cmd::FontBigger), M::Item(Cmd::FontSmaller), M::Item(Cmd::ResetView), M::Sep,
        M::Check(Cmd::ShowAddr), M::Check(Cmd::HotKeys)]),
    ("Projekt", &[M::Item(Cmd::Build), M::Item(Cmd::Run), M::Item(Cmd::Debug), M::Sep,
        M::Item(Cmd::BuildActive), M::Item(Cmd::RunActive), M::Item(Cmd::DebugActive), M::Sep, M::Item(Cmd::ProjectSettings), M::Sep, M::Item(Cmd::LiveImage)]),
    ("Debug", &[M::Item(Cmd::DbgCont), M::Item(Cmd::DbgPause), M::Item(Cmd::DbgStepInto), M::Item(Cmd::DbgStepOver), M::Item(Cmd::DbgStepOut), M::Item(Cmd::DbgStop), M::Sep, M::Item(Cmd::DbgMemory), M::Item(Cmd::DbgEmulator)]),
    ("Fenster", &[M::Item(Cmd::StudioDetach), M::Item(Cmd::StudioEmbed)]),
    ("Hilfe", &[M::Item(Cmd::Help), M::Item(Cmd::Shortcuts), M::Sep,
        M::Sub("Konfigurationsassistent", &[M::Item(Cmd::SetupAll), M::Item(Cmd::SetupRoms), M::Item(Cmd::SetupEmulators), M::Item(Cmd::SetupHardware), M::Item(Cmd::SetupDesign), M::Sep, M::Item(Cmd::SetupExport), M::Item(Cmd::SetupImport), M::Item(Cmd::SetupReset)]), M::Sep, M::Item(Cmd::RomSetup), M::Item(Cmd::RomFolder), M::Item(Cmd::RomCheck), M::Sep, M::Item(Cmd::Feedback), M::Item(Cmd::About)]),
];

/// Studios, von denen ein Befehl abhängt (mindestens eines muss im Release sein); leer = unabhängig
pub fn needs_studio(c: Cmd) -> Vec<&'static str> {
    use Cmd::*;
    match c {
        NewMedia | Export | StudioDetach | StudioEmbed | HotKeys => crate::studios::media().map(|s| s.id).collect(),
        StdCharset | ImportRom | ImportSheet => vec!["char"],
        _ => vec![],
    }
}

/// Erscheint der Befehl in dieser Auswahl (Menü und Befehlspalette)? Befehle abgewählter Studios fehlen ganz.
pub fn visible_with(c: Cmd, f: &vc_ide::features::Features) -> bool { let n = needs_studio(c); (n.is_empty() || n.iter().any(|s| f.studio(s))) && link_ok(c, f) }

/// Befehle, die eine Anbindung brauchen: Live-Bild die Ultimate, Debugger-Befehle eine Anbindung mit Debugger, die Einrichtung von Emulatoren
/// bzw. Hardware mindestens eine davon
fn link_ok(c: Cmd, f: &vc_ide::features::Features) -> bool {
    use Cmd::*;
    match c {
        LiveImage => f.any_live_image(),
        Debug | DebugActive | DbgCont | DbgPause | DbgStepInto | DbgStepOver | DbgStepOut | DbgStop | DbgMemory | DbgEmulator | WorkDebug => f.any_debugger(),
        SetupEmulators => vc_ide::emulators::available_in(f).iter().any(|_| true),
        SetupHardware => vc_ide::emulators::devices().iter().any(|d| f.link(&d.id)),
        _ => true,
    }
}

/// Einträge ohne abgewählte Befehle; doppelte, führende und hängende Trennlinien entfallen
pub fn visible_items(items: &[M], f: &vc_ide::features::Features) -> Vec<M> {
    let mut out: Vec<M> = vec![];
    for m in items {
        match m {
            M::Item(c) | M::Check(c) if !visible_with(*c, f) => {}
            M::Sep => { if !matches!(out.last(), None | Some(M::Sep)) { out.push(*m); } }
            _ => out.push(*m),
        }
    }
    while matches!(out.last(), Some(M::Sep)) { out.pop(); }
    out
}

#[cfg(test)]
/// Steht der Befehl in dieser Auswahl irgendwo im Menü (auch in Untermenüs, die beim Aufbau gefiltert werden)?
pub fn menu_has(f: &vc_ide::features::Features, c: Cmd) -> bool {
    fn has(items: &[M], f: &vc_ide::features::Features, c: Cmd) -> bool {
        visible_items(items, f).iter().any(|m| match m { M::Item(x) | M::Check(x) => *x == c, M::Sub(_, sub) => has(sub, f, c), _ => false })
    }
    MENUS.iter().any(|(_, items)| has(items, f, c))
}

/// Die Menüleiste dieser Auswahl; Menüs ohne Einträge entfallen
pub fn visible_menus(f: &vc_ide::features::Features) -> Vec<(&'static str, Vec<M>)> {
    MENUS.iter().map(|(n, items)| (*n, visible_items(items, f))).filter(|(_, v)| !v.is_empty()).collect()
}

/// Zustand eines Häkchens im Menü
pub fn checked(c: Cmd, s: &MenuState) -> bool {
    use Cmd::*;
    match c {
        SchemeDark => s.scheme == 0, SchemeLight => s.scheme == 1, SchemeSystem => s.scheme == 2,
        HotKeys => s.hotkeys, ShowAddr => s.show_addr,
        ToggleSide => s.side, ToggleInspector => s.inspector, ToggleFooter => s.footer,
        WorkCode => !s.debug_wp, WorkDebug => s.debug_wp,
        _ => false,
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::{Cmd, MenuState};
    use muda::accelerator::{Accelerator, Code, Modifiers};
    use muda::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};
    use std::collections::HashMap;
    use std::sync::mpsc::{channel, Receiver};

    pub struct Native {
        _menu: Menu,
        ids: HashMap<MenuId, Cmd>,
        items: Vec<(Cmd, MenuItem)>,
        checks: Vec<(Cmd, CheckMenuItem)>,
        rx: Receiver<Cmd>,
        last: Option<MenuState>,
        /// Untermenü „Zuletzt geöffnet“: Einträge, Namen und die Kennungen, die der Klick-Handler kennt
        recent: Option<Submenu>, recent_items: Vec<MenuItem>, recent_names: Vec<String>, shared: std::sync::Arc<std::sync::Mutex<HashMap<MenuId, Cmd>>>,
    }

    /// Menütext aus der Tabelle; Befehle ohne eigenes Menükürzel nennen ihr Kürzel in Klammern
    fn text_of(c: Cmd, has_accelerator: bool) -> String {
        let i = super::info(c);
        if i.keys.is_empty() || has_accelerator { i.label.to_string() } else { format!("{}  ({})", i.label, super::keys_text(i.keys, true)) }
    }

    fn cmd_key(m: Modifiers, c: Code) -> Option<Accelerator> { Some(Accelerator::new(Some(m), c)) }

    /// Sammelt Einträge samt ihrer Befehle
    #[derive(Default)]
    struct B { recent: Option<Submenu>, ids: HashMap<MenuId, Cmd>, items: Vec<(Cmd, MenuItem)>, checks: Vec<(Cmd, CheckMenuItem)> }
    /// Tastenkürzel der Tabelle als Menükürzel: nur Cmd-Kürzel mit Buchstabe, Komma, Eingabe oder Rückstrich (F-Tasten und Plus/Minus verarbeitet egui selbst)
    fn accel(c: Cmd) -> Option<Accelerator> {
        let keys = super::info(c).keys;
        if !keys.starts_with("Cmd+") { return None; }
        let mut m = Modifiers::empty();
        let mut parts: Vec<&str> = keys.split('+').collect();
        let last = parts.pop()?;
        for p in parts { match p { "Cmd" => m |= Modifiers::SUPER, "Umschalt" => m |= Modifiers::SHIFT, "Alt" => m |= Modifiers::ALT, _ => return None } }
        let code = match last {
            "," => Code::Comma, "Enter" => Code::Enter, "\\" => Code::Backslash,
            l if l.len() == 1 && l.as_bytes()[0].is_ascii_uppercase() => match l { "A" => Code::KeyA, "B" => Code::KeyB, "F" => Code::KeyF, "L" => Code::KeyL, "J" => Code::KeyJ, "K" => Code::KeyK, "N" => Code::KeyN, "O" => Code::KeyO, "P" => Code::KeyP, "S" => Code::KeyS, "W" => Code::KeyW, "Y" => Code::KeyY, _ => return None },
            _ => return None,
        };
        Some(Accelerator::new(Some(m), code))
    }
    impl B {
        fn fill(&mut self, sub: &Submenu, items: &[super::M]) {
            for m in items {
                match m {
                    super::M::Item(c) if matches!(c, Cmd::About | Cmd::Settings) => {}   // stehen im Programmmenü
                    super::M::Item(c) => self.item(sub, *c, accel(*c)),
                    super::M::Check(c) => self.check(sub, *c, accel(*c)),
                    super::M::Sep => { let _ = sub.append(&PredefinedMenuItem::separator()); }
                    super::M::Recent => { let r = Submenu::new("Zuletzt geöffnet", true); let _ = sub.append(&r); self.recent = Some(r); }
                    super::M::Sub(name, items) => { let s = Submenu::new(*name, true); self.fill(&s, &super::visible_items(items, &vc_ide::features::features())); let _ = sub.append(&s); }
                }
            }
        }
        fn item(&mut self, sub: &Submenu, c: Cmd, acc: Option<Accelerator>) {
            let it = MenuItem::new(text_of(c, acc.is_some()), true, acc);
            self.ids.insert(it.id().clone(), c); let _ = sub.append(&it); self.items.push((c, it));
        }
        fn check(&mut self, sub: &Submenu, c: Cmd, acc: Option<Accelerator>) {
            let it = CheckMenuItem::new(text_of(c, acc.is_some()), true, false, acc);
            self.ids.insert(it.id().clone(), c); let _ = sub.append(&it); self.checks.push((c, it));
        }
    }

    impl Native {
        /// Menüleiste aufbauen und in die Anwendung einhängen; `ctx` wird bei jedem Klick neu gezeichnet
        pub fn new(ctx: &eframe::egui::Context) -> Native {
            let menu = Menu::new();
            let mut b = B::default();
            let cmd = Modifiers::SUPER;
            // Programmmenü
            let app = Submenu::new("VintageCoder", true);
            b.item(&app, Cmd::About, None);
            let _ = app.append(&PredefinedMenuItem::separator());
            b.item(&app, Cmd::Settings, cmd_key(cmd, Code::Comma));
            let _ = app.append(&PredefinedMenuItem::separator());
            let _ = app.append(&PredefinedMenuItem::services(None));
            let _ = app.append(&PredefinedMenuItem::separator());
            let _ = app.append(&PredefinedMenuItem::hide(None));
            let _ = app.append(&PredefinedMenuItem::hide_others(None));
            let _ = app.append(&PredefinedMenuItem::show_all(None));
            let _ = app.append(&PredefinedMenuItem::separator());
            let _ = app.append(&PredefinedMenuItem::quit(Some("VintageCoder beenden")));
            let _ = menu.append(&app);
            // übrige Menüs aus der Tabelle (Ausschneiden, Kopieren, Einfügen bleiben bei egui: vordefinierte Einträge würden die Tastenkürzel abfangen)
            for (title, items) in &super::visible_menus(&vc_ide::features::features()) {
                let sub = Submenu::new(*title, true);
                b.fill(&sub, items);
                if *title == "Fenster" {
                    let _ = sub.append(&PredefinedMenuItem::separator());
                    let _ = sub.append(&PredefinedMenuItem::minimize(None));
                    let _ = sub.append(&PredefinedMenuItem::maximize(Some("Zoomen")));
                    let _ = sub.append(&PredefinedMenuItem::fullscreen(None));
                    sub.set_as_windows_menu_for_nsapp();
                }
                let _ = menu.append(&sub);
            }
            menu.init_for_nsapp();
            // Klicks in einen Kanal; der Handler weckt die Oberfläche
            let (tx, rx) = channel();
            let shared = std::sync::Arc::new(std::sync::Mutex::new(b.ids.clone())); let map = shared.clone(); let ctx2 = ctx.clone();
            MenuEvent::set_event_handler(Some(move |e: MenuEvent| { let c = map.lock().ok().and_then(|m| m.get(e.id()).copied()); if let Some(c) = c { let _ = tx.send(c); ctx2.request_repaint(); } }));
            Native { _menu: menu, ids: b.ids, items: b.items, checks: b.checks, rx, last: None, recent: b.recent, recent_items: vec![], recent_names: vec![], shared }
        }

        /// Seit dem letzten Aufruf angeklickte Befehle
        pub fn poll(&self) -> Vec<Cmd> { self.rx.try_iter().collect() }

        /// Liste „Zuletzt geöffnet“ setzen (nur bei Änderung)
        pub fn set_recent(&mut self, names: &[String]) {
            if self.recent_names == names { return; }
            let Some(sub) = &self.recent else { return };
            for it in self.recent_items.drain(..) { let _ = sub.remove(&it); if let Ok(mut m) = self.shared.lock() { m.remove(it.id()); } }
            for (i, n) in names.iter().enumerate() {
                let it = MenuItem::new(n, true, None);
                if let Ok(mut m) = self.shared.lock() { m.insert(it.id().clone(), Cmd::Recent(i as u8)); }
                let _ = sub.append(&it); self.recent_items.push(it);
            }
            if names.is_empty() { let it = MenuItem::new("Keine", false, None); let _ = sub.append(&it); self.recent_items.push(it); }
            self.recent_names = names.to_vec();
        }

        /// Aktivierung und Haken setzen (nur bei Änderung)
        pub fn sync(&mut self, s: MenuState) {
            if self.last == Some(s) { return; }
            self.last = Some(s);
            for (c, it) in &self.items {
                let on = super::enabled(*c, &s);
                it.set_enabled(on);
            }
            for (c, it) in &self.checks { it.set_checked(super::checked(*c, &s)); }
            let _ = &self.ids;
        }
    }
}

#[cfg(target_os = "macos")]
pub use imp::Native;

/// Auf anderen Systemen gibt es keine native Menüleiste
#[cfg(not(target_os = "macos"))]
pub struct Native;
#[cfg(not(target_os = "macos"))]
impl Native {
    pub fn new(_ctx: &eframe::egui::Context) -> Native { Native }
    pub fn poll(&self) -> Vec<Cmd> { vec![] }
    pub fn sync(&mut self, _s: MenuState) {}
    pub fn set_recent(&mut self, _names: &[String]) {}
}


#[cfg(test)]
mod tests {
    use super::*;

    fn all_items(items: &[M], out: &mut Vec<Cmd>) {
        for m in items { match m { M::Item(c) | M::Check(c) => out.push(*c), M::Sep | M::Recent => {}, M::Sub(_, i) => all_items(i, out) } }
    }

    #[test]
    fn menuestruktur_folgt_dem_konzept() {
        let names: Vec<&str> = MENUS.iter().map(|m| m.0).collect();
        assert_eq!(names, ["Datei", "Bearbeiten", "Ansicht", "Projekt", "Debug", "Fenster", "Hilfe"]);
    }

    #[test]
    fn jeder_menuebefehl_steht_in_der_befehlstabelle_und_einmal_im_menue() {
        let mut v = vec![];
        for (_, items) in MENUS { all_items(items, &mut v); }
        for c in &v { assert!(COMMANDS.iter().any(|i| i.cmd == *c), "{c:?} fehlt in COMMANDS"); }
        for c in &v { assert_eq!(v.iter().filter(|x| *x == c).count(), 1, "{c:?} doppelt im Menü"); }
    }

    #[test]
    fn debug_befehle_nur_mit_sitzung() {
        let s = MenuState { project: true, idle: true, can_debug: true, ..Default::default() };
        assert!(!enabled(Cmd::DbgStop, &s) && !enabled(Cmd::DbgStepInto, &s));
        let d = MenuState { debugging: true, halted: false, ..s };
        assert!(enabled(Cmd::DbgStop, &d) && enabled(Cmd::DbgPause, &d) && !enabled(Cmd::DbgStepInto, &d));
        assert!(enabled(Cmd::DbgStepInto, &MenuState { halted: true, ..d }));
    }

    #[test]
    fn haekchen_der_zonen_und_arbeitsplaetze() {
        let s = MenuState { side: true, footer: false, debug_wp: true, ..Default::default() };
        assert!(checked(Cmd::ToggleSide, &s) && !checked(Cmd::ToggleFooter, &s));
        assert!(checked(Cmd::WorkDebug, &s) && !checked(Cmd::WorkCode, &s));
    }
}
