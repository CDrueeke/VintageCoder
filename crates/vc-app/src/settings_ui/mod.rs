// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungsseite: Registerkarten Allgemein, Zielsysteme und Hardware, Toolchain, Editor, Tastenkürzel. Die Seite „Zielsysteme und
//! Hardware“ ist im Master-Detail-Muster gebaut (links die Zielsysteme, rechts Emulatoren und Hardware des gewählten Systems).
//!
//! Aufbau: `state` (Auswahl, Proben, Hintergrund-Ergebnisse), `services` (Hintergrund-Threads über `vc_ide::platform`),
//! `sidebar`/`header`/`emulators`/`hardware`/`footer` (Ansichten der Zielsystem-Seite), `pages` (übrige Seiten) und `widgets` (Bausteine).
//! Das Modell selbst (Regeln für Standard-Emulator, Startziel, Migration) steht in `vc_ide::targets`; die Ansichten ändern nur den Entwurf `draft`,
//! gespeichert wird in `app.rs`. Darstellung und Editor-Einstellungen wirken sofort (nicht über den Entwurf).
use eframe::egui::{self, RichText};
use vc_ide::settings::Settings;

mod args;
pub(crate) mod emulators;
mod footer;
pub(crate) mod hardware;
mod header;
mod install;
pub(crate) mod pages;
pub mod roms;
pub(crate) mod services;
mod sidebar;
pub(crate) mod state;
mod widgets;

pub use pages::Env;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Act { None, Save, Revert, OpenProjectSettings }

#[derive(PartialEq, Eq, Clone, Copy)]
enum Page { General, Targets, Roms, Toolchain, Editor, Keys }

pub struct SettingsEd {
    pub draft: Settings,
    pub saved: Settings,
    ui: state::UiState,
    page: Page,
    roms: roms::RomsUi,
}

impl SettingsEd {
    pub fn new(s: Settings) -> SettingsEd {
        // ältere Dateien (nach Programmen gegliedert) werden in Zielsysteme überführt; das gilt als gespeicherter Stand
        let mut s = s; s.migrate_legacy();
        // Selbsttest (Screenshots): VC_SET_PAGE=general|toolchain|editor|keys
        let page = match std::env::var("VC_SET_PAGE").ok().filter(|_| std::env::var("VC_SHOT").is_ok()).as_deref() { Some("general") => Page::General, Some("toolchain") => Page::Toolchain, Some("roms") => Page::Roms, Some("editor") => Page::Editor, Some("keys") => Page::Keys, _ => Page::Targets };
        let mut ed = SettingsEd { saved: s.clone(), draft: s, ui: state::UiState::new(), page, roms: Default::default() };
        // Selbsttest (Screenshots): VC_SET_ARGS=1 füllt den ersten Emulator mit Beispielparametern
        if std::env::var("VC_SHOT").is_ok() && std::env::var("VC_SET_ARGS").is_ok() {
            if let Some(e) = ed.draft.targets.iter_mut().find(|t| t.system == "c64").and_then(|t| t.emulators.first_mut()) {
                for (k, v) in [("warp", "on"), ("filter", "1"), ("sound", "off"), ("moncommands", "{symbols}"), ("initbreak", "0x{start}"), ("speed", "200")] { e.opts.insert(k.into(), v.into()); }
                e.args = "-VICIIvsync -gibtsnicht +warp".into();
            }
            ed.draft.assembler.opts.insert("illegals".into(), "deny".into());
            ed.draft.assembler.opts.insert("include".into(), "{project_dir}/inc".into());
            ed.draft.assembler.args = "-D DEBUG=1 -D LEVEL=0x10 --listing x".into();
        }
        ed
    }
    pub fn dirty(&self) -> bool { self.draft != self.saved }
    /// Überschreibungen des geöffneten Projekts melden (`None`: kein Projekt)
    pub fn set_project(&mut self, ov: Option<Vec<vc_ide::targets::ProjectOverride>>) { if self.ui.project != ov { self.ui.project = ov; } }
    /// System, dessen Überschreibung entfernt werden soll (einmalig)
    pub fn take_clear_override(&mut self) -> Option<String> { self.ui.clear_override.take() }
}

pub fn show(ui: &mut egui::Ui, ed: &mut SettingsEd, env: &mut Env) -> Act {
    if ed.ui.poll() { ui.ctx().request_repaint_after(std::time::Duration::from_millis(200)); }
    let mut act = Act::None;
    let pages = [(Page::General, "Allgemein"), (Page::Targets, "Zielsysteme und Hardware"), (Page::Roms, "ROMs"), (Page::Toolchain, "Toolchain"), (Page::Editor, "Editor"), (Page::Keys, "Tastenkürzel")];
    egui::Panel::top("set_tabs").show_inside(ui, |ui| {
        let tabs: Vec<crate::shell::ModeTab> = pages.iter().map(|(p, l)| crate::shell::ModeTab { label: l, active: ed.page == *p, open: false, count: if *p == Page::Targets { Some(ed.draft.targets.len()) } else { None }, hint: String::new() }).collect();
        if let Some(i) = crate::shell::mode_bar(ui, &tabs) { ed.page = pages[i].0; }
        if ed.dirty() && !matches!(ed.page, Page::Targets | Page::Toolchain) { ui.label(RichText::new("● Ungespeicherte Änderungen bei „Zielsysteme und Hardware“ oder „Toolchain“").small().color(crate::theme::warn())); }
    });
    if ed.page == Page::Targets { return show_targets(ui, ed); }
    // Die Toolchain-Seite hat veränderbare Werte (Assembler-Optionen): gleiche Fußzeile wie die Zielsysteme
    if ed.page == Page::Toolchain { let dirty = ed.dirty(); egui::Panel::bottom("set_footer_tc").show_inside(ui, |ui| { let a = footer::show(ui, dirty); if a != Act::None { act = a; } }); }
    egui::CentralPanel::default().show_inside(ui, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(8.0);
            ui.set_max_width(if ed.page == Page::Roms { 1240.0 } else { 820.0 });
            match ed.page {
                Page::General => pages::general(ui, env, &mut ed.draft.help, &mut ed.saved.help),
                Page::Toolchain => { let a = pages::toolchain(ui, env, &mut ed.draft.assembler, &mut ed.ui.focus); if a != Act::None { act = a; } }
                Page::Editor => pages::editor(ui, env),
                Page::Keys => pages::keys(ui),
                Page::Roms => roms::show(ui, &mut ed.roms, &mut ed.draft.roms, &mut ed.saved.roms),
                Page::Targets => {}
            }
        });
    });
    act
}

fn show_targets(ui: &mut egui::Ui, ed: &mut SettingsEd) -> Act {
    // Auswahl gültig halten
    if ed.draft.target(&ed.ui.system).is_none() { ed.ui.system = ed.draft.targets.first().map(|t| t.system.clone()).unwrap_or_default(); }
    let dirty = ed.dirty();
    let mut act = Act::None;
    egui::Panel::bottom("set_footer").show_inside(ui, |ui| act = footer::show(ui, dirty));
    egui::Panel::left("set_nav").default_size(270.0).size_range(210.0..=380.0).show_inside(ui, |ui| sidebar::show(ui, &mut ed.draft, &mut ed.ui));
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if ed.draft.target(&ed.ui.system).is_none() {
            ui.add_space(20.0);
            ui.label(RichText::new("Kein Zielsystem ausgewählt. Links unten lässt sich eines hinzufügen.").weak());
            return;
        }
        let mut area = egui::ScrollArea::vertical().auto_shrink([false, false]);
        // Selbsttest (Screenshots): VC_SET_SCROLL=<Pixel> blättert die Seite
        if let Some(y) = std::env::var("VC_SET_SCROLL").ok().filter(|_| std::env::var("VC_SHOT").is_ok()).and_then(|v| v.parse::<f32>().ok()) { area = area.vertical_scroll_offset(y); }
        area.show(ui, |ui| {
            ui.add_space(6.0);
            header::show(ui, &mut ed.draft, &mut ed.ui);
            ui.add_space(10.0);
            emulators::show(ui, &mut ed.draft, &mut ed.ui);
            ui.add_space(10.0);
            hardware::show(ui, &mut ed.draft, &mut ed.ui);
        });
    });
    act
}
