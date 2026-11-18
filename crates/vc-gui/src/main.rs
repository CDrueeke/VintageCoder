// SPDX-License-Identifier: GPL-3.0-or-later
// Windows: GUI-Anwendung ohne Konsolenfenster (die CLI `vcc` bleibt Konsolenprogramm).
#![cfg_attr(windows, windows_subsystem = "windows")]
//! VintageCoder, neue Oberfläche (Iced), Stufe 1: Gerüst und Editor.
//! Aufruf: `vintagecoder [Projektordner]`. Elm-Aufbau: `App` (Modell), `Msg`, `update`, `view`, `subscription`.
mod aussen;
mod prettyzeile;
mod hexansicht;
mod bildcache;
mod tastatur;
mod telemetrie;
mod pfadfeld;
mod mdansicht;
mod basic2asm;
mod zahlen;
mod konfig;
mod mdhilfen;
mod zeichendlg;
mod hexdlg;
mod studioaktionen;
mod audio;
mod exportdlg;
mod palettestudio;
mod importdlg;
mod studio;
mod diskstudio;
mod screenstudio;
mod tilestudio;
mod bankstudio;
mod songstudio;
/// Ergebnis einer Song-Umwandlung als Nachricht (`Result` allein ist nicht `Clone`-pflichtig, hier eingepackt)
mod songimport_ergebnis { #[derive(Clone, Debug)] pub struct Erg(pub Result<(String, Vec<String>), String>); }
mod dbgview;
mod debug;
mod testlauf;
mod einstellungen;
mod setui;
mod romui;
mod zielui;
mod argsui;
mod startui;
mod neuprojekt;
mod kopierdlg;
mod projeinst;
mod assistent;
mod infodlg;
mod textdatei;
mod gruppen;
mod lsp;
#[cfg(target_os = "macos")]
mod native;
mod project;

use einstellungen::{Einstellungen, Schema};
use iced::widget::{rich_text, span, button, column, container, image, row, scrollable, space, stack, text, text_input};
use iced::{keyboard, Alignment, Background, Border, Color, Element, Length, Subscription, Task, Theme};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use vc_gui_editor::core::{Editor, Pos};
use vc_gui_editor::widget::{editor_view, EdMsg};
use vc_gui_ui::befehle::{self, Braucht, Cmd};
use vc_gui_ui::texte as t;
use vc_gui_ui::bereich::{self as bw, Aktionen};
use vc_gui_ui::kontext::{self, Art, Zeile};
use vc_gui_ui::theme::Pal;
use vc_ide_model::layout::{Anzeige, Bereich, Gemerkt, Layout};

const ICON: &[u8] = include_bytes!("../../../branding/vintagecoder-icon.png");
const WORT_FUER_DUNKEL: &[u8] = include_bytes!("../../../branding/vintagecoder-wordmark-light.png");
const WORT_FUER_HELL: &[u8] = include_bytes!("../../../branding/vintagecoder-wordmark-dark.png");

#[derive(Clone, Debug, PartialEq)]
pub enum Tab { Datei(String), Einstellungen, Start, Projekt, /** Hilfe-Thema im Dokumentenfeld (Kennung; leer = Übersicht) */ Hilfe(String) }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seite { Dateien, Suche }
/// Reiter, der gerade gezogen wird (Maus gedrückt auf einem Reiter)
/// Ziehen im Dateibaum: Quelle, ob Ordner, ob die Maus sich über eine andere Zeile bewegt hat, und der Zielordner darunter
struct BaumZug { quelle: String, ordner: bool, bewegt: bool, ziel: Option<String> }
struct Ziehen { tab: Tab, start: Option<iced::Point>, jetzt: iced::Point }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BA { Min, Max, Wieder }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelSeite { Gliederung, Hilfe, Zeichen, Debug, Karte, Emulator }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZeichenWahl { Steuer(usize), Code(u8) }
/// Zustand des Panels „Speicherkarte“
pub struct KartenZustand { pub sel: u8, pub filter: &'static str, pub preset: Option<String>, pub banking: Option<vc_target::Banking>, pub target_id: String, pub facts: Option<vc_ide_model::memmap_facts::Facts> }
impl Default for KartenZustand { fn default() -> Self { KartenZustand { sel: 0, filter: "all", preset: None, banking: None, target_id: String::new(), facts: None } } }
/// Rückfrage vor einer Handlung, die eine laufende Debug-Sitzung beenden würde (nie stillschweigend)
#[derive(Clone, Debug, PartialEq)] pub enum Folge { Beenden, Projekt(PathBuf) }
#[derive(Clone, Debug, PartialEq)] pub enum Rueckfrage { TestDebug(PathBuf, String), DebugNeu, ScratchWeg(String), Kachelkarte(String), KonfigImport(PathBuf, String), KonfigReset }
/// Was nach dem Bauen geschieht
#[derive(Clone, Copy, PartialEq, Debug)] pub enum Danach { Nichts, Start, Debug }
/// Ergebnis des letzten erfolgreichen Baus (Startdatei, Ziel, Symbole für den Debugger)
#[derive(Clone, Debug)] pub struct BauInfo { pub target: String, pub artifact: std::path::PathBuf, pub symbols: HashMap<String, i64>, pub run: Option<u16>, pub zeilen: Vec<(String, usize, u16, usize)>, pub segmente: Vec<(u32, usize)> }
impl BauInfo {
    /// Quelldatei und Zeile (1-basiert) zu einer Adresse
    pub fn zeile_bei(&self, pc: u16) -> Option<(&str, usize)> { self.zeilen.iter().find(|(_, _, a, l)| pc >= *a && (pc as usize) < *a as usize + (*l).max(1)).map(|(f, n, _, _)| (f.as_str(), *n)) }
    /// Adresse zu Datei und Zeile (1-basiert)
    pub fn adresse_von(&self, datei: &str, zeile: usize) -> Option<u16> { self.zeilen.iter().find(|(f, n, _, _)| f == datei && *n == zeile).map(|(_, _, a, _)| *a) }
    /// Marke zu einer Adresse (für die Anzeige im Disassembler)
    pub fn marke_bei(&self, a: u16) -> Option<&str> { self.symbols.iter().filter(|(n, v)| **v == a as i64 && !n.contains('.')).map(|(n, _)| n.as_str()).min() }
}
#[derive(Clone, Debug)]
pub struct Problem { pub datei: String, pub zeile: usize, pub text: String, pub fehler: bool }
#[derive(Clone, Debug)]
pub enum Eintrag { Befehl(Cmd), Datei(String), Projekt(PathBuf, String), /** Aktion des Studios im aktiven Reiter: Name, Kürzel, Nachricht */ Studio(String, String, Box<Msg>) }

#[derive(Debug, Clone)]
pub enum Msg {
    Start, A11yBereit, SystemSchema(iced::theme::Mode),
    Ed(EdMsg), Menue(String), Befehl(Cmd), Taste(keyboard::Key, keyboard::Modifiers), /** Taste mit physischer Position (Klaviertasten gelten nach der Position, nicht nach der Beschriftung) */ TasteP(keyboard::Key, keyboard::Modifiers, keyboard::key::Physical),
    OeffneDatei(String), Reiter(usize), Schliesse(usize), Schublade(usize),
    ImportAn(usize, bool), ImportKopie(usize), ImportName(usize, String), ImportOrdner(usize, String), ImportKonflikt(usize, vc_ide::importfiles::Conflict), ImportOk, ImportAbbruch, ExportFormat(String), ExportName(String), ExportAdresse(String), ExportTab(String), ExportAus(exportdlg::ExportAus), PaletteAkt(String, palettestudio::PAkt), PngGewaehlt(String, Option<PathBuf>), StudioTick, StudioMaus(String, studio::Maus), StudioAkt(String, studio::Aktion), DisHaltepunkt(u16), DebugSpalte(usize), TrennerStart, TrennerMaus(iced::Point), TrennerLos, RueckfrageJa, RueckfrageNein, EmuBild, EmuAuto(bool), EmuLive(bool), EmuQuelle(usize), EmuText(String), EmuSenden, TestAlle, TestFehler, TestStopp, TestBackend, TestFilter(String), TestNurFehler(bool), TestWahl(PathBuf, String), TestDebug(PathBuf, String), TestQuelle(String, usize), TestTick, SnapErkennen, SnapName(String), SnapSprites(bool), SnapKacheln(bool), SnapRom(bool), SnapSpeichern, ZielMenue, KopfMenue, KopfWahl(Cmd), Set(setui::SetAkt), StartSeite(startui::StartAkt), ScratchTick, ScratchRest(u8), SongScroll(String, f32, f32, f32, f32), NeuProj(neuprojekt::NpAkt), Kopieren(kopierdlg::KopAkt), ProjEinst(projeinst::PeAkt), Wiz(assistent::WizAkt), Info(infodlg::InfoAkt), FensterZu, BeendenAntwort(u8), ZielWahl(String), KarteSeite(u8), KarteFilter(&'static str), KartePreset(String), DebugTick, MemText(String), MemGo, BeobText(String), BeobAdd, BeobDel(usize), PunktDel(u32), BauFertig(Result<serde_json::Value, String>), Gestartet(Result<String, String>), StartFehler(Vec<String>), GotoProblem(usize),
    PaletteFrage(String), PaletteAuswahl(i32), PaletteLos(usize),
    SeiteWahl(Seite), BaumKlapp(String), BaumFilter(String), BaumNeuLaden, BaumAlleZu, SucheFrage(String), SucheLos, SucheFertig(Vec<project::Fund>), GotoFund(usize),
    FindFrage(String), ErsetzFrage(String), FindSchritt(i32), ErsetzeEins, ErsetzeAlle, FindGrossKlein, FindZu,
    GotoText(String), GotoLos, GotoZeile(usize),
    SetSchema(Schema), SetZoom(f32), SetSchrift(f32), SetUmbruch(bool), SetStudioTasten(bool), SetDiagnose(bool), SetPretty(bool), Bildschirm(iced::window::Id, Option<iced::Size>), KontextMsg(Box<Msg>), MdMehr, KonfigWahl(konfig::KArt), KonfigPfad(konfig::KArt, Option<PathBuf>), KonfigResetFrage, SplashZu, SetSplash(bool), Zahl(zahlen::ZAkt), MdUmschalten(String), PfadWaehlen(pfadfeld::PfadArt, String, pfadfeld::Setzer), PfadGewaehlt(pfadfeld::PfadArt, pfadfeld::Setzer, Option<PathBuf>), SetTakt(bool), SetLayout(einstellungen::Tastaturlayout), SetKlavierTasten(bool),
    PanelWahl(PanelSeite), StreifenKlick(PanelSeite), BereichAkt(Bereich, BA), BereichAuf(Bereich), SpalteAuf(Seite), HilfeSuche(String), HilfeOeffne(String), HilfeZurueck, ZeichenSuche(String), ZeichenEinfuegen(ZeichenWahl),
    EdIn(bool, EdMsg), GruppeKlick(bool), SongKonv(String, bool), SongDatei(String, bool, Option<PathBuf>), SongImpName(String), SongImpSub(i32), SongImpSek(i32), SongImpOk, SongImpAbbruch, SongImpFertig(songimport_ergebnis::Erg), SongTick, SongAkt(String, songstudio::GAkt), SongMaus(String, songstudio::GMaus), BankAkt(String, bankstudio::BAkt), BankMaus(String, bankstudio::BMaus), TileAkt(String, tilestudio::TAkt), TileMaus(String, tilestudio::TMaus), ScreenAkt(String, screenstudio::SAkt), ScreenMaus(String, screenstudio::SMaus), DiskAkt(String, diskstudio::DAkt), DiskPfad(String, diskstudio::DPfad, Option<PathBuf>), DiskDateien(String, Vec<PathBuf>), DiskFertig(String, Result<String, String>), ReiterIn(bool, usize), SchliesseIn(bool, usize), TabDruck(bool, usize), TabUeber(Option<(bool, usize)>), Maus(iced::Point), MausLos, Fenster(iced::Size),
    KontextAuf(Art, iced::Point, Option<String>), KontextReiter(bool, usize, iced::Point), KontextWahl(Cmd), KontextZu, MenueAuf(String), SetMenueleiste(bool), Einfuegen(Option<String>),
    MdBildGewaehlt(Option<PathBuf>), MdLink(String), HilfeImReiter, HexVorschlag(hexdlg::HAkt), Zeichen(zeichendlg::ZAkt), ExportKachel(exportdlg::KachelAkt), HexAkt(String, hexansicht::HAkt), BaumZugStart(String, bool), BaumZugUeber(String), BaumZugLos(String), BaumZugAbbruch, BaumOrdnerWahl(String), BaumNeu(String, NeuArt), BaumOrdner(String), BaumArtWahl(NeuArt), BaumProfil(usize), BaumUmbenennen, BaumLoeschen, BaumText(String), BaumOk, BaumAbbruch,
    Abgelegt(PathBuf), Geoeffnet(Option<PathBuf>), Lsp(lsp::Antwort), Nichts,
}

/// Empfänger der Sprachdienst-Antworten (die Subscription holt ihn einmal ab)
static LSP_EMPFAENGER: std::sync::Mutex<Option<std::sync::mpsc::Receiver<lsp::Antwort>>> = std::sync::Mutex::new(None);

struct Palette { offen: bool, frage: String, auswahl: usize }
struct FindLeiste { offen: bool, ersetzen: bool, frage: String, mit: String, gross_klein: bool, info: String }
/// Art der neuen Datei im Dialog „Neue Datei …“
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NeuArt { Asm, Basic, Text, Notiz, Studio(vc_ide_model::project::Group), /** leere Diskette: Kennung `d64`, `d71`, `d81`, `atr` */ Diskette(&'static str) }
impl NeuArt {
    fn endung(self) -> &'static str { match self { NeuArt::Asm => ".asm", NeuArt::Basic => ".bas", NeuArt::Text => ".txt", NeuArt::Notiz => ".md", NeuArt::Studio(_) => "", NeuArt::Diskette(k) => match k { "d71" => ".d71", "d81" => ".d81", "atr" => ".atr", _ => ".d64" } } }
    fn name(self) -> &'static str { match self { NeuArt::Asm => t::ART_ASM, NeuArt::Basic => t::ART_BASIC, NeuArt::Text => t::ART_TEXT, NeuArt::Notiz => t::ART_NOTIZ, NeuArt::Studio(g) => g.title(), NeuArt::Diskette(k) => match k { "d71" => "Diskette D71", "d81" => "Diskette D81", "atr" => "Diskette ATR", _ => "Diskette D64" } } }
}
enum BaumArt { Neu { ordner: String, art: NeuArt, profile: Vec<(String, String)>, profil: usize }, Ordner(String), Umbenennen(String), Loeschen(String), SpeichernUnter(String) }
/// Offenes Kontextmenü: Art, Ort im Fenster, Ziel (Datei im Baum), Tastaturauswahl
struct Kontext { art: Art, pos: iced::Point, ziel: Option<String>, sel: usize }
/// Ordner in der Auswahlliste der Anlege-Dialoge
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] struct OrdnerWahl(String);
impl std::fmt::Display for OrdnerWahl { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { if self.0.is_empty() { write!(f, "(Projektordner)") } else { write!(f, "{}/", self.0) } } }
struct BaumDialog { art: BaumArt, text: String, fehler: String }
#[derive(Default)]
struct HilfeZustand { suche: String, treffer: Vec<vc_ide_model::helpstate::Result>, doc: Option<vc_ide_model::helpstate::Doc>, hist: Vec<vc_ide_model::helpstate::Doc>, themen: Vec<vc_ide_model::helpstate::Topic> }
struct GotoLeiste { offen: bool, text: String }

pub struct App {
    einst: Einstellungen, sys_dunkel: bool, root: PathBuf, data: PathBuf, proj: vc_ide_model::project::Project, namen: Vec<String>,
    tabs: Vec<Tab>, aktiv: usize, editoren: HashMap<String, Editor>, schublade: usize, log: Vec<String>, probleme: Vec<Problem>, baut: bool,
    palette: Palette, seite: Seite, baum_ziel: String, baum_zu: std::collections::BTreeSet<String>, baum_filter: String, baum_datei: Option<PathBuf>, suche_frage: String, funde: Vec<project::Fund>, sucht: bool, suche_fertig: bool,
    find: FindLeiste, goto: GotoLeiste, gliederung: RefCell<(String, u64, Vec<(usize, String)>)>, logos: [image::Handle; 3], unbenannt: usize,
    lsp: lsp::Lsp, danach: Danach, info: Option<BauInfo>, run_on: String, ziel_menue: bool, kopf_menue: bool, textmeta: HashMap<String, textdatei::Meta>, lesefehler: HashSet<String>, scratch: Vec<(String, bool)>, scratch_n: u32, scratch_datei: Option<PathBuf>, scratch_stand: String, scratch_rest: Option<Vec<vc_ide_model::scratchsave::Entry>>, scratch_kept: Vec<vc_ide_model::scratchsave::Entry>, neu_projekt: Option<neuprojekt::NeuProj>, kopier: Option<kopierdlg::KopierDlg>, pe: Option<projeinst::PeZ>, wiz: Option<assistent::Wiz>, upd: infodlg::InfoZ, wechsel_ohne_speichern: bool, beenden_frage: Option<(Vec<String>, Folge)>, start: startui::StartZ, satz: setui::Satz, paletten: HashMap<String, vc_ide_model::palettedoc::PaletteDoc>, pal_hex: String, import: Option<importdlg::ImportDialog>, export: Option<exportdlg::ExportDialog>, letzte_asm: Option<String>, studios: HashMap<String, studio::PixelStudio>, farben: studio::Farben, debug_spalte: usize, trenner: bool, reg_alt: Option<[u16; 6]>, reg_neu: Option<[u16; 6]>, rueckfrage: Option<Rueckfrage>, emu_bild: Option<(u64, image::Handle, usize, usize)>, emu_ms: f32, emu_auto: bool, emu_live: bool, emu_text: String, tests: testlauf::TestLauf, test_debug: bool, snap_name: String, snap_sprites: bool, snap_tiles: bool, snap_rom: bool, karte: KartenZustand, dbg: Option<debug::Debugger>, run_to: Option<(u16, bool)>, snap: debug::Snap, bp_an: HashSet<u16>, dbg_fehler: u32, dbg_notizen: usize, mem_text: String, beob: Vec<String>, beob_text: String, punkt_text: String, lsp_gesendet: HashMap<String, u64>, diagnosen: HashMap<String, Vec<lsp::Diag>>,
    andere: Option<(Vec<Tab>, usize)>, fokus_rechts: bool, ziehen: Option<Ziehen>, ueber_tab: Option<(bool, usize)>, fenster: iced::Size,
    kontext: Option<Kontext>, ziel: Option<String>, def_req: u64,
    layout: Layout, fokus_bereich: Bereich, marken: vc_ide_model::cursors::Marks, marken_datei: Option<PathBuf>,
    baum: Option<BaumDialog>, papierkorb: bool,
    panel_seite: PanelSeite, hilfe: HilfeZustand, zeichen_suche: String, ed_fokus: u64, ed_aktiv: bool, zeichen_cache: RefCell<(bool, u8, Vec<vc_ide_model::charpanel::Entry>)>,
    /** letzter Befehl aus Menü oder Tastatur (Befehl, kam aus dem Menü, Zeit): dasselbe Kürzel kann unter macOS als Menüeintrag UND als Tastenereignis ankommen; das zweite wird verworfen */ /** Prettyprint: bearbeitete Zeile (Datei, Zeile, Zeilenzahl davor), die beim Verlassen formatiert wird */ pretty_zeile: Option<(String, usize, usize)>, /** kurzer Hinweis in der Statusleiste (bis zur nächsten Taste) */ hinweis: String, /** Menüleiste im Fenster auch unter macOS zeigen (Tests und Galerie) */ menue_test: bool, /** nur Bilder: Kürzel in der Schreibweise von Windows und Linux zeigen */ kuerzel_windows: bool, doppel: Option<(Cmd, bool, std::time::Instant)>, doppel_ms: u128,
    vorschlag_noetig: bool, vorschlag_req: u64, vorschlag_fuer: (u64, vc_gui_editor::core::Pos), hover_req: u64, lsp_gen: u32, exportiert: Vec<String>, hexe: HashMap<String, hexansicht::HexDoc>, baum_zug: Option<BaumZug>, /** ein Vorhörton klingt: Tick für die Klaviatur-Anzeige */ probe_aktiv: bool, /** Startbild sichtbar */ splash: bool, zahlen: Option<zahlen::ZahlenDlg>, /** per Tastatur oder Klick gewählte Zeile des Dateibaums (rel; Ordner und Dateien) */ baum_sel: Option<String>, /** geparste Markdown-Dokumente (nach Inhalt) */ md_cache: std::cell::RefCell<HashMap<u64, std::rc::Rc<vc_md::Doc>>>, /** Markdown-Dateien, die als Quelltext (nicht als Vorschau) offen sind */ md_quelltext: HashSet<String>, md_als: HashSet<String>, zeichen: Option<zeichendlg::ZeichenDlg>, hex: Option<hexdlg::HexDlg>, /** wie viele Blöcke große Markdown-Texte zeigen */ md_grenze: usize, /** QWERTY-Zeichen der gerade gedrückten physischen Taste (nur während der Verarbeitung einer Taste) */ phys: Option<char>, layout_gelernt: Option<einstellungen::Tastaturlayout>, layout_system: Option<einstellungen::Tastaturlayout>, disks: HashMap<String, vc_ide_model::diskdoc::DiskDoc>, screens: HashMap<String, screenstudio::ScreenStudio>, tilesets: HashMap<String, tilestudio::TileStudio>, banken: HashMap<String, bankstudio::BankStudio>, songs: HashMap<String, songstudio::SongStudio>, audio: audio::Audio, audio_song: Option<String>, song_import: Option<songstudio::SongImport>,
}

/// Ordner der Hardware-Datenbank (`data/`) wie bei `vcc`: Umgebungsvariable `VC_DATA`, sonst ab dem Ort des Programms (installierte App, Quellbaum),
/// sonst ab dem aktuellen Ordner. Kein zur Übersetzungszeit eingebrannter Pfad.
fn daten_ordner() -> PathBuf {
    if let Some(p) = std::env::var_os("VC_DATA").filter(|p| !p.is_empty()) { return PathBuf::from(p) }
    std::env::current_exe().ok().and_then(|e| vc_ide::find_data_dir(&e)).or_else(|| std::env::current_dir().ok().and_then(|d| vc_ide::find_data_dir(&d))).unwrap_or_default()
}
/// Platzhalter-Ordner für den Start ohne Projekt (leer; die Startseite zeigt keine Dateien)
fn ohne_projekt_ordner() -> PathBuf { std::env::temp_dir().join("vintagecoder-ohne-projekt") }
fn leerer_ordner() -> PathBuf { let d = ohne_projekt_ordner(); let _ = std::fs::create_dir_all(&d); d }

/// Ereignisse des Fensters für die Programmlogik. Tasten kommen nur an, wenn kein Widget sie verbraucht hat – außer Esc: ein Texteingabefeld
/// mit Fokus (Befehlsleiste, Assistent) verbraucht Esc selbst, trotzdem soll es Leiste und Assistent schließen.
fn fenster_ereignis(e: iced::Event, status: iced::event::Status) -> Option<Msg> {
    match e {
        iced::Event::Window(iced::window::Event::FileDropped(p)) => Some(Msg::Abgelegt(p)),
        iced::Event::Window(iced::window::Event::Resized(s)) => Some(Msg::Fenster(s)),
        iced::Event::Window(iced::window::Event::CloseRequested) => Some(Msg::FensterZu),
        iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, physical_key, .. })
            if status == iced::event::Status::Ignored || matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) => Some(Msg::TasteP(key, modifiers, physical_key)),
        _ => None,
    }
}

impl App {
    fn neu() -> (App, Task<Msg>) { Self::neu_aus(std::env::args().collect(), cfg!(debug_assertions)) }
    /// `args` wie die Kommandozeile (erstes Element: Programmname); `entwicklung`: ohne Projektangabe das Beispiel „prallball“ aus dem Quellbaum öffnen
    /// (nur für Entwickler und Tests). Sonst erscheint die Startseite: ohne Projekt, ohne Eintrag in „Zuletzt geöffnet“.
    fn neu_aus(args: Vec<String>, entwicklung: bool) -> (App, Task<Msg>) {
        // Testlauf: nie Prozesse starten (Finder, Papierkorb über osascript, Emulatoren): Sicherung für alle Module, die `VC_TEST_NO_SPAWN` beachten
        #[cfg(test)] std::env::set_var("VC_TEST_NO_SPAWN", "1");
        // Testlauf: „Zuletzt geöffnet“ nie in die echte Konfiguration schreiben, sondern je Test-Thread in eine eigene Wegwerfdatei
        #[cfg(test)] if !vc_ide_model::recent::umgeleitet() { vc_ide_model::recent::umleiten(Some(std::env::temp_dir().join(format!("vcgui-recent-test-{}-{:?}.json", std::process::id(), std::thread::current().id())))) }
        let data = daten_ordner();
        let start = args.iter().skip(1).filter(|a| !a.starts_with("--")).map(PathBuf::from).find(|a| a.exists()).and_then(|a| project::ziel_fuer(&a));
        let ohne_projekt = start.is_none() && !entwicklung;
        let (root, start_datei, start_vcprj) = match start { Some(z) => (z.root, z.datei, z.vcprj), None if entwicklung => (data.parent().map(|p| p.join("examples/c64/prallball")).unwrap_or_default(), None, None), None => (leerer_ordner(), None, None) };
        let (proj, start_fehler) = if ohne_projekt { (project::laden(&root), None) } else { match project::pruefen(&root, start_vcprj) { Ok(p) => (p, None), Err(e) => (project::laden(&root), Some(e)) } };
        let namen = project::dateien(&proj);
        let (lsp_start, lsp_rx) = lsp::Lsp::start(&root, &data); *LSP_EMPFAENGER.lock().unwrap() = Some(lsp_rx);
        let mut app = App { einst: Einstellungen::laden(), sys_dunkel: true, data: data.clone(), root, proj, namen, tabs: vec![], aktiv: 0, editoren: HashMap::new(), schublade: 0, log: vec![t::BEREIT.into()], probleme: vec![], baut: false,
            palette: Palette { offen: false, frage: String::new(), auswahl: 0 }, seite: Seite::Dateien, baum_ziel: String::new(), baum_zu: Default::default(), baum_filter: String::new(), baum_datei: if cfg!(test) { None } else { vc_ide_model::treestate::file() }, suche_frage: String::new(), funde: vec![], sucht: false, suche_fertig: false,
            find: FindLeiste { offen: false, ersetzen: false, frage: String::new(), mit: String::new(), gross_klein: false, info: String::new() }, goto: GotoLeiste { offen: false, text: String::new() },
            pretty_zeile: None, hinweis: String::new(), menue_test: std::env::var_os("VC_MENUE_IM_FENSTER").is_some_and(|v| v == "1"), kuerzel_windows: false, doppel: None, doppel_ms: if cfg!(test) { 0 } else { 300 }, gliederung: RefCell::new((String::new(), 0, vec![])), logos: [image::Handle::from_bytes(WORT_FUER_DUNKEL), image::Handle::from_bytes(WORT_FUER_HELL), image::Handle::from_bytes(ICON)], unbenannt: 0, danach: Danach::Nichts, info: None, run_on: String::new(), ziel_menue: false, kopf_menue: false, textmeta: HashMap::new(), lesefehler: HashSet::new(), scratch: vec![], scratch_n: 0, scratch_datei: if cfg!(test) { None } else { vc_ide_model::scratchsave::file() }, scratch_stand: String::new(), scratch_rest: None, scratch_kept: vec![], neu_projekt: None, kopier: None, pe: None, wiz: None, upd: Default::default(), wechsel_ohne_speichern: false, beenden_frage: None, start: Default::default(), satz: setui::Satz::neu(), paletten: HashMap::new(), pal_hex: String::new(), import: None, export: None, letzte_asm: None, studios: HashMap::new(), farben: studio::Farben::laden(&data), debug_spalte: 0, trenner: false, reg_alt: None, reg_neu: None, rueckfrage: None, emu_bild: None, emu_ms: 0.0, emu_auto: true, emu_live: false, emu_text: String::new(), tests: testlauf::TestLauf::default(), test_debug: false, snap_name: "momentaufnahme".into(), snap_sprites: true, snap_tiles: false, snap_rom: true, karte: KartenZustand::default(), dbg: None, run_to: None, snap: debug::Snap::default(), bp_an: HashSet::new(), dbg_fehler: 0, dbg_notizen: 0, mem_text: "0400".into(), beob: vec![], beob_text: String::new(), punkt_text: String::new(), lsp: lsp_start, lsp_gesendet: HashMap::new(), diagnosen: HashMap::new(), panel_seite: PanelSeite::Gliederung, hilfe: HilfeZustand::default(), zeichen_suche: String::new(), ed_fokus: 0, ed_aktiv: true, zeichen_cache: RefCell::new((false, 255, vec![])), andere: None, fokus_rechts: false, ziehen: None, ueber_tab: None, fenster: iced::Size::new(1280.0, 800.0), kontext: None, ziel: None, def_req: 0, layout: Layout::aus_gemerkt(Gemerkt { seitenleiste_minimiert: false, schublade_minimiert: false, panel_minimiert: true }), fokus_bereich: Bereich::Dokument, marken: Default::default(), marken_datei: if cfg!(test) { None } else { vc_ide_model::cursors::file() }, baum: None, papierkorb: vc_ide_model::fileops::can_trash(), vorschlag_noetig: false, vorschlag_req: 0, vorschlag_fuer: (0, Default::default()), hover_req: 0, lsp_gen: 0, exportiert: vec![], disks: HashMap::new(), hexe: HashMap::new(), baum_zug: None, probe_aktiv: false, splash: false, zahlen: None, baum_sel: None, md_cache: Default::default(), md_quelltext: HashSet::new(), md_als: HashSet::new(), zeichen: None, hex: None, md_grenze: mdansicht::BLOECKE, phys: None, layout_gelernt: None, layout_system: tastatur::system_layout(), screens: HashMap::new(), tilesets: HashMap::new(), banken: HashMap::new(), songs: HashMap::new(), audio: audio::Audio::neu(), audio_song: None, song_import: None };
        app.marken = app.marken_datei.as_ref().map(|f| vc_ide_model::cursors::load_from(f, &app.root)).unwrap_or_default();
        app.baum_zu = app.baum_datei.as_ref().map(|f| vc_ide_model::treestate::load_from(f, &app.root)).unwrap_or_default();
        app.layout = Layout::aus_gemerkt(Gemerkt { seitenleiste_minimiert: !app.einst.seitenleiste, schublade_minimiert: !app.einst.schublade, panel_minimiert: !app.einst.panel });
        app.panel_seite = match app.einst.panel_reiter { 1 => PanelSeite::Hilfe, 2 => PanelSeite::Zeichen, _ => PanelSeite::Gliederung };
        app.start_laden(); if !ohne_projekt { app.zuletzt_merken() }
        // Ersteinrichtung: weder wizard.json noch settings.json → der Assistent öffnet sich von selbst (nie im Test, nie mit VC_NO_WIZARD)
        if !cfg!(test) && !vc_ide::setup::auto_start_suppressed() { if let Some(p) = app.wiz_pfade() { if vc_ide::setup::start_state(&p) == vc_ide::setup::Start::FirstRun { app.wiz_oeffnen(None) } } }
        if !cfg!(test) && vc_ide_model::scratchsave::enabled() { if let Some(f) = &app.scratch_datei { let e = vc_ide_model::scratchsave::load_from(f, vc_ide_model::scratchsave::now_secs()); if !e.is_empty() { app.scratch_rest = Some(e) } } }
        if let Some(e) = start_fehler { app.log.push(e) }
        if let Some(f) = start_datei.or_else(|| project::startprogramm(&app.proj)) { app.oeffne(&f) }
        (app, Task::done(Msg::Start))
    }
    /// Jedes geöffnete Projekt (Startargument, Dialog, Startseite, Ziehen und Ablegen) kommt in „Zuletzt geöffnet“; im Testlauf nur in die umgeleitete Wegwerfdatei
    fn zuletzt_merken(&mut self) {
        #[cfg(test)] if !vc_ide_model::recent::umgeleitet() { return }
        let mut r = vc_ide_model::recent::load(Some(&self.data)); vc_ide_model::recent::touch(&mut r, &self.root, &self.proj.name, Some(&self.data)); vc_ide_model::recent::save(&r); self.start.recent = r;
    }
    /// Panel mit Reiter `p` ausklappen und die Wahl merken
    fn panel_oeffnen(&mut self, p: PanelSeite) -> Task<Msg> {
        self.layout.aufklappen(Bereich::Panel); self.panel_seite = p; self.einst.panel_reiter = match p { PanelSeite::Gliederung | PanelSeite::Debug | PanelSeite::Karte | PanelSeite::Emulator => 0, PanelSeite::Hilfe => 1, PanelSeite::Zeichen => 2 }; self.layout_merken();
        match p { PanelSeite::Hilfe => { if self.hilfe.themen.is_empty() { self.hilfe.themen = vc_ide_model::helpstate::topics() } self.text_fokus("hilfesuche") } PanelSeite::Zeichen => self.text_fokus("zeichensuche"), PanelSeite::Gliederung | PanelSeite::Debug | PanelSeite::Karte | PanelSeite::Emulator => Task::none() }
    }
    /// Gemerkten Teil des Layouts (minimiert ja/nein) in die Einstellungen schreiben
    fn layout_merken(&mut self) { let g = self.layout.gemerkt(); self.einst.seitenleiste = !g.seitenleiste_minimiert; self.einst.schublade = !g.schublade_minimiert; self.einst.panel = !g.panel_minimiert; self.einst.speichern() }
    /// Breite des Panels: während einer Debug-Sitzung breiter (Disassembler, Speicher)
    fn panel_breite(&self) -> f32 { if self.panel_seite == PanelSeite::Emulator { 460.0 } else if self.panel_seite == PanelSeite::Karte { 380.0 } else { 260.0 } }
    fn panel_offen(&self) -> bool { self.layout.offen(Bereich::Panel) }
    fn streifen_zustand(&self) -> Vec<(String, bool)> { [(t::GLIEDERUNG, PanelSeite::Gliederung), (t::HILFE_TITEL, PanelSeite::Hilfe), (t::ZEICHEN, PanelSeite::Zeichen), (t::KARTE, PanelSeite::Karte)].iter().map(|(n, p)| (n.to_string(), self.panel_offen() && self.panel_seite == *p)).collect() }
    /// Bereiche für den Bildschirmleser: (Name, Zustand, Knopfnamen)
    fn bereiche_zustand(&self) -> Vec<(String, String, Vec<String>)> {
        [(Bereich::Seitenleiste, t::SPALTE), (Bereich::Dokument, t::DOKUMENTE), (Bereich::Schublade, t::SCHUBLADE), (Bereich::Panel, t::PANEL)].iter().map(|(b, n)| {
            let a = self.layout.anzeige(*b); let (mi, ma) = bw::knopfnamen(n, a);
            (n.to_string(), bw::zustandstext(a).to_string(), if self.layout.kann_minimieren(*b) && a != Anzeige::Minimiert { vec![mi, ma] } else { vec![ma] })
        }).collect()
    }
    /// Schmale Leiste am rechten Rand mit den Panel-Reitern; bleibt auch bei eingeklapptem Panel sichtbar
    fn streifen(&self, pal: Pal) -> Element<'_, Msg> {
        let knopf = |symbol: &'static str, p: PanelSeite| -> Element<Msg> { let auf = self.panel_offen() && self.panel_seite == p; button(text(symbol).size(15).color(if auf { pal.text } else { pal.mute }).center()).width(30).height(30).on_press(Msg::StreifenKlick(p)).style(move |_, st| button::Style { background: if auf { Some(pal.sel.into()) } else if matches!(st, button::Status::Hovered) { Some(pal.panel2.into()) } else { None }, border: Border { radius: 6.0.into(), color: if auf { pal.accent } else { Color::TRANSPARENT }, width: if auf { 1.0 } else { 0.0 } }, text_color: pal.text, ..Default::default() }).into() };
        container({ let mut s = column![knopf("≡", PanelSeite::Gliederung), knopf("?", PanelSeite::Hilfe), knopf("Ω", PanelSeite::Zeichen), knopf("▦", PanelSeite::Karte)].spacing(4); if self.dbg.is_some() && self.snap.can_frame { s = s.push(knopf("◉", PanelSeite::Emulator)) } s }.align_x(Alignment::Center)).width(38).height(Length::Fill).padding([8, 4]).style(move |_| flaeche(pal.panel, pal.line)).into()
    }
    /// Unter dieser Breite (Oberflächeneinheiten) legt sich das ausgeklappte Panel über das Dokument, statt es zu verschieben
    fn schmal(&self) -> bool { self.fenster.width / self.einst.zoom < 900.0 }
    fn hilfe_ziel(&self) -> String { vc_ide_model::helpstate::target_of(&self.proj.systems) }
    fn hilfe_sprache(&self) -> &'static str { match self.name().and_then(|n| self.editoren.get(&n)).map(|e| e.lang) { Some(vc_ide_model::lang::Lang::Basic) => "basic", _ => "asm" } }
    fn atari(&self) -> bool { self.proj.systems.first().is_some_and(|s| s.starts_with("atari")) }
    /// Steuerzeichen und Codes der Familie des Projekts (aus der gemeinsamen Namensquelle), je Sprache zwischengespeichert
    fn zeichen_eintraege(&self, lang: vc_ide_model::lang::Lang) -> Vec<vc_ide_model::charpanel::Entry> {
        let (atari, l) = (self.atari(), lang as u8); let mut c = self.zeichen_cache.borrow_mut();
        if c.0 != atari || c.1 != l || c.2.is_empty() { *c = (atari, l, vc_ide_model::charpanel::entries(atari, lang)) } c.2.clone()
    }
    /// Zeichen an der Cursorstelle einfügen, in der Form, die dort passt (Name in Zeichenketten, `CHR$` im BASIC …)
    fn zeichen_einfuegen(&mut self, w: ZeichenWahl) {
        use vc_ide_model::{charesc, charpanel};
        let Some(n) = self.name() else { self.log.push(t::BAUM_KEINE_DATEI.into()); return };
        let Some(e) = self.editoren.get(&n) else { return };
        let (lang, atari) = (e.lang, self.atari()); let line = &e.lines[e.cursor.line]; let before: String = line.chars().take(e.cursor.col).collect();
        let in_text = lang == vc_ide_model::lang::Lang::Asm && charesc::asm_text_line(line) && before.matches('"').count() % 2 == 1;
        let spot = charpanel::spot(lang, &before, in_text);
        let txt = match w { ZeichenWahl::Steuer(i) => { let list = self.zeichen_eintraege(lang); charpanel::default_form(lang, spot).zip(list.get(i)).map(|(f, en)| charpanel::render(f, en)) } ZeichenWahl::Code(c) => charpanel::code_text(atari, 0, lang, Some(spot), c) };
        match txt { Some(x) => { if let Some(e) = self.editoren.get_mut(&n) { e.insert(&x) } self.editor_fokus() } None => self.log.push(t::ZEICHEN_NICHT_HIER.into()) }
    }
    /// Studio-Arten, für die das Projekt Profile hat: (Art, Profile (Kennung, Name))
    fn studio_profile(&self, g: vc_ide_model::project::Group) -> Vec<(String, String)> {
        let sys = vc_media::project::project_systems(&self.root, &self.data);
        vc_media::load_profiles(&self.data).map(|l| l.into_iter().filter(|p| (sys.is_empty() || sys.contains(&p.target)) && vc_ide_model::project::Group::of_kind(&p.kind) == g).map(|p| (p.id, p.name)).collect()).unwrap_or_default()
    }
    fn studio_arten(&self) -> Vec<vc_ide_model::project::Group> {
        use vc_ide_model::project::Group::*; [Sprite, Charset, Screen, Tiles, Palette, Song, Play].into_iter().filter(|g| !self.studio_profile(*g).is_empty()).collect()
    }
    /// Art im Dialog „Neue Datei …“ wählen (bei Studio-Dateien mit der Profilliste des Projekts)
    fn neu_art_waehlen(&mut self, neu: NeuArt) {
        let profile = if let NeuArt::Studio(g) = neu { self.studio_profile(g) } else { vec![] };
        if let Some(BaumDialog { art: BaumArt::Neu { ordner, art, profile: p, profil }, .. }) = &mut self.baum {
            *art = neu; *p = profile; *profil = 0; if neu == NeuArt::Notiz && ordner.is_empty() { *ordner = "notes".into() }
        }
    }
    /// Dialog des Dateibaums ausführen (Neu, Ordner, Umbenennen, Löschen); Fehler bleiben im Dialog stehen
    fn baum_ausfuehren(&mut self) {
        let Some(b) = self.baum.take() else { return };
        let fehler = |s: &mut App, mut b: BaumDialog, e: String| { b.fehler = e; s.baum = Some(b) };
        match b.art {
            BaumArt::Neu { ref ordner, art, ref profile, profil } => {
                let (ordner, name) = (ordner.clone(), b.text.trim().to_string());
                let ergebnis: Result<(String, bool), String> = if let NeuArt::Diskette(k) = art {
                    let name = if name.contains('.') { name } else { format!("{name}{}", art.endung()) };
                    let rel = if ordner.is_empty() { name.clone() } else { format!("{ordner}/{name}") };
                    let stem: String = name.rsplit_once('.').map_or(name.as_str(), |x| x.0).to_uppercase().chars().filter(|c| c.is_ascii_alphanumeric() || *c == ' ').take(16).collect();
                    let ziel = project::pfad(&self.root, &rel);
                    if ziel.exists() { Err(format!("{rel} gibt es schon")) } else {
                        vc_ide_model::diskdoc::DiskDoc::neu(vc_ide_model::diskdoc::format_aus_kennung(k), if stem.is_empty() { "DISK" } else { &stem }, "01", None).and_then(|mut d| { if let Some(o) = ziel.parent() { std::fs::create_dir_all(o).map_err(|e| e.to_string())? } d.speichern_unter(ziel.clone()); match d.note { Some((false, m)) => Err(m), _ => Ok((rel, true)) } })
                    }
                } else if let NeuArt::Studio(_) = art {
                    match profile.get(profil) { Some((id, _)) => vc_media::project::create(&self.root, &self.data, &name, id, &ordner).map(|v| (v["path"].as_str().unwrap_or("").to_string(), false)), None => Err("Für dieses Projekt gibt es dafür kein Profil.".into()) }
                } else {
                    let name = if name.contains('.') { name } else { format!("{name}{}", art.endung()) };
                    let rel = if ordner.is_empty() { name } else { format!("{ordner}/{name}") };
                    vc_ide_model::fileops::create_file(&self.root, &rel).map(|r| (r, true))
                };
                match ergebnis {
                    Ok((rel, oeffnen)) => { self.projekt_neu_laden(); self.log.push(format!("Angelegt: {rel}")); let hat_studio = rel.ends_with(".vcsprite") || rel.ends_with(".vcpmg") || rel.ends_with(".vcchar") || rel.ends_with(".vcpal");
                        if oeffnen || hat_studio { self.oeffne(&rel) } else { self.log.push("Bildschirm-, Kachel- und Tracker-Dateien öffnen sich mit ihren Studios (folgt).".into()) } }
                    Err(e) => fehler(self, b, e),
                }
            }
            BaumArt::Ordner(ref ordner) => { let rel = if ordner.is_empty() { b.text.trim().to_string() } else { format!("{ordner}/{}", b.text.trim()) }; match vc_ide_model::fileops::create_dir(&self.root, &rel) { Ok(r) => { self.projekt_neu_laden(); self.log.push(format!("Ordner angelegt: {r}/")) } Err(e) => fehler(self, b, e) } }
            BaumArt::SpeichernUnter(ref n) => { let n = n.clone(); let rel = b.text.trim().to_string(); match self.scratch_speichern_unter(&n, &rel) { Ok(()) => {} Err(e) => fehler(self, b, e) } }
            BaumArt::Umbenennen(ref alt) => { let alt = alt.clone(); let name = b.text.trim().to_string();
                if name.contains('/') || name.contains('\\') { fehler(self, b, "Nur der Name, kein Pfad: Verschieben geht per Ziehen im Baum.".into()) } else {
                    let neu_rel = match alt.rsplit_once('/') { Some((o, _)) => format!("{o}/{name}"), None => name.clone() };
                    let ist_ordner = project::pfad(&self.root, &alt).is_dir();
                    let r = if ist_ordner { vc_ide_model::fileops::rename_dir(&mut self.proj, &alt, &neu_rel) } else { vc_ide_model::fileops::rename_file(&mut self.proj, &alt, &neu_rel).map(|n| (n.clone(), vec![(alt.clone(), n)])) };
                    match r {
                        Ok((neu, paare)) => {
                            for (a, n) in &paare { self.datei_verschoben(a, n) }
                            self.projekt_neu_laden(); self.log.push(format!("Umbenannt: {alt} → {neu}")); self.markiere()
                        }
                        Err(e) => fehler(self, b, e),
                    } } }
            BaumArt::Loeschen(ref rel) => { let rel = rel.clone(); match vc_ide_model::fileops::delete_file(&mut self.proj, &rel, self.papierkorb) {
                Ok(korb) => {
                    self.tab_entfernen(&Tab::Datei(rel.clone()));
                    self.editoren.remove(&rel); if self.lsp_gesendet.remove(&rel).is_some() { self.lsp.senden(lsp::Cmd::Close { uri: lsp::uri(&self.root, &rel) }) } self.diagnosen.remove(&rel);
                    self.projekt_neu_laden(); self.log.push(format!("{} {rel}", if korb { "In den Papierkorb gelegt:" } else { "Gelöscht:" })); self.markiere()
                }
                Err(e) => fehler(self, b, e),
            } }
        }
    }
    /// Projekt wechseln: Ordner, `.vcprj` oder Datei (→ Projekt der nächsten `.vcprj` darüber, sonst Einzeldatei). Fehler stehen im Protokoll
    pub(crate) fn projekt_oeffnen(&mut self, p: PathBuf) -> bool {
        let Some(z) = project::ziel_fuer(&p) else { self.oeffne_pfad(p); return true };
        if self.dbg.is_some() || self.baut { self.log.push("Projekt wechseln: erst die Debug-Sitzung beziehungsweise den Bau beenden.".into()); self.schublade_zeigen(); return false }
        if z.root == self.root && z.vcprj == self.proj.file || z.root == self.root && z.vcprj.is_none() { self.zuletzt_merken(); if let Some(d) = z.datei.or_else(|| project::startprogramm(&self.proj)) { self.oeffne(&d) } return true }
        let proj = match project::pruefen(&z.root, z.vcprj.clone()) { Ok(pr) => pr, Err(e) => { self.log.push(e); self.schublade_zeigen(); return false } };
        if !self.wechsel_ohne_speichern { for n in self.alle_namen() { if !self.ist_scratch(&n) { self.speichere_datei(&n) } } }
        let notizen: Vec<(String, Editor, bool)> = self.scratch.iter().filter_map(|(n, nt)| self.editoren.remove(n).map(|e| (n.clone(), e, *nt))).collect();
        if let Some(f) = &self.marken_datei { let _ = vc_ide_model::cursors::save_to(f, &self.root, &self.marken); }
        self.baum_zustand_sichern();
        self.root = z.root; self.namen = project::dateien(&proj); self.proj = proj; self.exportiert = project::exportierte(&self.root);
        self.pe = None; self.kopier = None; self.tabs.clear(); self.andere = None; self.aktiv = 0; self.editoren.clear(); for (n, e, _) in notizen { self.editoren.insert(n.clone(), e); self.tabs.push(Tab::Datei(n)); } self.studios.clear(); self.paletten.clear(); self.disks.clear(); self.hexe.clear(); self.screens.clear(); self.tilesets.clear(); self.banken.clear(); self.songs.clear(); self.audio.stoppen(); self.audio_song = None; self.diagnosen.clear(); self.lsp_gesendet.clear();
        self.probleme.clear(); self.funde.clear(); self.suche_fertig = false; self.info = None; self.letzte_asm = None; self.kontext = None; self.baum = None; self.export = None; self.import = None; self.ziel = None; self.bp_an.clear(); self.beob.clear();
        self.marken = self.marken_datei.as_ref().map(|f| vc_ide_model::cursors::load_from(f, &self.root)).unwrap_or_default();
        self.baum_zu = self.baum_datei.as_ref().map(|f| vc_ide_model::treestate::load_from(f, &self.root)).unwrap_or_default(); self.baum_filter.clear();
        let (l, rx) = lsp::Lsp::start(&self.root, &self.data); self.lsp = l; self.lsp_gen += 1; if cfg!(test) { drop(rx) } else { *LSP_EMPFAENGER.lock().unwrap() = Some(rx) }
        self.zuletzt_merken();
        self.log.push(format!("Projekt geöffnet: {}", self.proj.name));
        if let Some(d) = z.datei.or_else(|| project::startprogramm(&self.proj)) { self.oeffne(&d) }
        true
    }
    fn schublade_zeigen(&mut self) { self.schublade = 0; self.layout.aufklappen(Bereich::Schublade) }
    fn projekt_neu_laden(&mut self) { self.proj = project::laden(&self.root); self.namen = project::dateien(&self.proj); self.exportiert = project::exportierte(&self.root); }
    fn dunkel(&self) -> bool { match self.einst.schema { Schema::Dunkel | Schema::Kontrast => true, Schema::Hell => false, Schema::System => self.sys_dunkel } }
    fn pal(&self) -> Pal { if self.einst.schema == Schema::Kontrast { Pal::hoch() } else if self.dunkel() { Pal::dark() } else { Pal::light() } }
    /// Gruppe auf der Seite `rechts` zur fokussierten machen (tauscht sie in `tabs`/`aktiv`; der übrige Code kennt nur die fokussierte Gruppe)
/// Fokus folgt dem Klick: die Gruppe, in der `rel` als aktiver Reiter sichtbar ist, wird aktive Gruppe und bekommt den Fokus (kein Textfeld/Editor behält ihn);
    /// so wirken Knöpfe und Kürzel immer auf das Studio, in dem geklickt wurde
    fn fokus_auf(&mut self, rel: &str) {
        let hier = matches!(self.tabs.get(self.aktiv), Some(Tab::Datei(n)) if n == rel);
        let dort = matches!(self.andere.as_ref().and_then(|a| a.0.get(a.1)), Some(Tab::Datei(n)) if n == rel);
        if !hier && dort { let r = !self.fokus_rechts; self.gruppe_fokus(r) }
        self.ed_aktiv = false; self.fokus_bereich = Bereich::Dokument;
    }
    fn gruppe_fokus(&mut self, rechts: bool) {
        if self.andere.is_none() || self.fokus_rechts == rechts { return }
        let a = self.andere.take().unwrap(); let eigene = (std::mem::replace(&mut self.tabs, a.0), std::mem::replace(&mut self.aktiv, a.1));
        self.andere = Some(eigene); self.fokus_rechts = rechts; self.markiere();
    }
    /// (links, rechts) als Paare herausnehmen; `gruppen_setzen` stellt sie wieder ein
    fn gruppen_nehmen(&mut self) -> (gruppen::Paar<Tab>, Option<gruppen::Paar<Tab>>) {
        let eigene = gruppen::Paar { tabs: std::mem::take(&mut self.tabs), aktiv: self.aktiv }; let andere = self.andere.take().map(|(tabs, aktiv)| gruppen::Paar { tabs, aktiv });
        match (andere, self.fokus_rechts) { (None, _) => (eigene, None), (Some(a), true) => (a, Some(eigene)), (Some(a), false) => (eigene, Some(a)) }
    }
    fn gruppen_setzen(&mut self, links: gruppen::Paar<Tab>, rechts: Option<gruppen::Paar<Tab>>, fokus_rechts: bool) {
        match rechts {
            None => { self.tabs = links.tabs; self.aktiv = links.aktiv; self.andere = None; self.fokus_rechts = false }
            Some(r) => { let (f, a) = if fokus_rechts { (r, links) } else { (links, r) }; self.tabs = f.tabs; self.aktiv = f.aktiv; self.andere = Some((a.tabs, a.aktiv)); self.fokus_rechts = fokus_rechts }
        }
        self.markiere();
    }
    fn tab_verschieben(&mut self, tab: Tab, nach_rechts: bool, vor: Option<usize>) {
        let (mut l, mut r) = self.gruppen_nehmen(); let f = gruppen::verschiebe(&mut l, &mut r, &tab, nach_rechts, vor); self.gruppen_setzen(l, r, f)
    }
    /// Aktiven Reiter in seiner Gruppe um einen Platz verschieben (Befehle ohne Maus)
    fn reiter_verschieben(&mut self, d: i32) {
        let (n, i) = (self.tabs.len(), self.aktiv); let Some(tab) = self.tabs.get(i).cloned() else { return };
        let ziel = i as i32 + d; if ziel < 0 || ziel >= n as i32 { return }
        let vor = if d < 0 { Some(i - 1) } else if i + 2 >= n { None } else { Some(i + 2) };
        let rechts = self.fokus_rechts; self.tab_verschieben(tab, rechts, vor)
    }
    /// Hover-Hinweis des Sprachdienstes als formatierte Karte (Überschrift, fett, Code, Aufzählung, Tabellen); leer, wenn es keinen gibt
    fn hover_karte<'a>(&self, pal: Pal, ed: &'a Editor) -> Element<'a, Msg> {
        let Some(h) = ed.hover.as_ref() else { return space::vertical().height(0).into() };
        md_karte(pal, &h.text)
    }
    fn reiter_breite(t: &Tab) -> f32 { let n = match t { Tab::Datei(n) => n.rsplit('/').next().unwrap_or(n).chars().count(), Tab::Einstellungen => 13, Tab::Start => 5, Tab::Projekt => 12, Tab::Hilfe(k) => k.chars().count().max(5) + 6 }; n as f32 * 7.0 + 50.0 + 4.0 }
    /// Verfügbare Breite der Reiterleiste einer Gruppe (Näherung in Oberflächeneinheiten), abzüglich der Knöpfe rechts
    fn leistenbreite(&self) -> f32 {
        let links = if self.layout.offen(Bereich::Seitenleiste) { 260.0 } else { 38.0 };
        let rechts = 38.0 + if self.layout.offen(Bereich::Panel) && !self.schmal() { self.panel_breite() } else { 0.0 };
        let gesamt = self.fenster.width / self.einst.zoom - links - rechts;
        (if self.andere.is_some() { gesamt / 2.0 } else { gesamt }) - 120.0
    }
    /// Welche Reiter sichtbar sind (von, bis ausschließlich): der aktive ist immer dabei, der Rest füllt die Leiste
    fn sichtbare_reiter(&self, tabs: &[Tab], aktiv: usize) -> (usize, usize) {
        let n = tabs.len(); if n == 0 { return (0, 0) } let aktiv = aktiv.min(n - 1);
        let avail = self.leistenbreite().max(200.0); let w = |i: usize| Self::reiter_breite(&tabs[i]);
        let (mut lo, mut hi, mut used) = (aktiv, aktiv + 1, w(aktiv));
        loop { if hi < n && used + w(hi) <= avail { used += w(hi); hi += 1 } else if lo > 0 && used + w(lo - 1) <= avail { lo -= 1; used += w(lo) } else { break } }
        (lo, hi)
    }
    /// Reiter an einer Stelle der Reiterleiste (Näherung aus der Textbreite), falls dort die Leiste liegt: (rechte Gruppe, Index; Einfügen davor)
    fn tab_unter(&self, x: f32, y: f32, x0: f32, x1: f32) -> Option<(bool, usize)> {
        if !(44.0..100.0).contains(&y) { return None }
        let rechts = self.andere.is_some() && x > (x0 + x1) / 2.0;
        let liste: Vec<&Tab> = if rechts == self.fokus_rechts { self.tabs.iter().collect() } else { self.andere.as_ref()?.0.iter().collect() };
        let aktiv = if rechts == self.fokus_rechts { self.aktiv } else { self.andere.as_ref()?.1 };
        let tabs_roh: Vec<Tab> = liste.iter().map(|t| (*t).clone()).collect(); let (lo, hi) = self.sichtbare_reiter(&tabs_roh, aktiv);
        let start = if rechts { (x0 + x1) / 2.0 } else { x0 };
        let mut cx = start + 8.0 + if lo > 0 { 52.0 } else { 0.0 };
        for i in lo..hi { let w = Self::reiter_breite(liste[i]); if x < cx + w / 2.0 { return Some((rechts, i)) } cx += w; }
        Some((rechts, hi))
    }
    fn teilen_umschalten(&mut self) {
        if self.andere.is_none() && self.tabs.len() < 2 { self.log.push(t::TEILEN_MIND_ZWEI.into()); return }
        let (mut l, mut r) = self.gruppen_nehmen(); let f = gruppen::teile_oder_vereine(&mut l, &mut r, self.fokus_rechts); self.gruppen_setzen(l, r, f)
    }
    /// Reiter überall entfernen (Datei gelöscht); leere Gruppen verschwinden
    fn tab_entfernen(&mut self, t: &Tab) {
        let (mut l, mut r) = self.gruppen_nehmen(); let f = self.fokus_rechts;
        for g in [Some(&mut l), r.as_mut()].into_iter().flatten() { if let Some(i) = g.tabs.iter().position(|x| x == t) { g.tabs.remove(i); g.aktiv = g.aktiv.min(g.tabs.len().saturating_sub(1)) } }
        if r.as_ref().is_some_and(|x| x.tabs.is_empty()) { r = None } if l.tabs.is_empty() { if let Some(x) = r.take() { l = x } }
        let fokus = f && r.is_some(); self.gruppen_setzen(l, r, fokus);
    }
    fn tab_oeffnen(&mut self, t: Tab) {
        if let Some(i) = self.andere.as_ref().and_then(|a| a.0.iter().position(|x| *x == t)) { self.gruppe_fokus(!self.fokus_rechts); self.aktiv = i; return }
 if let Some(i) = self.tabs.iter().position(|x| *x == t) { self.aktiv = i } else { self.tabs.push(t); self.aktiv = self.tabs.len() - 1 } }
    /// Ältere Kachelkarte (kind tilemap) in Kachelsatz und Bildschirm umwandeln: die alte Datei wird vorher unter `.vcbackup/<Zeit>/` gesichert
    fn kachelkarte_umwandeln(&mut self, rel: &str) {
        let alt = project::pfad(&self.root, rel);
        let sicherung = self.root.join(".vcbackup").join(format!("kachelkarte-{}", vc_ide_model::recent::now())).join(rel);
        if let Some(p) = sicherung.parent() { let _ = std::fs::create_dir_all(p); }
        if let Err(e) = std::fs::copy(&alt, &sicherung) { self.log.push(format!("{rel}: Sicherung fehlgeschlagen, nichts umgewandelt: {e}")); self.schublade_zeigen(); return }
        match vc_media::project::migrate_tiles_in(&self.root, &self.data, Some(rel), true) {
            Ok(bericht) => {
                for z in &bericht { self.log.push(z.clone()) } self.log.push(format!("Kachelkarte umgewandelt; Sicherung der alten Datei: {}", vc_core::paths::rel_path(&self.root, &sicherung)));
                self.projekt_neu_laden();
                let bank = rel.rsplit_once('.').map_or(rel.to_string(), |(a, _)| a.to_string()) + ".vcscreen"; if project::pfad(&self.root, &bank).is_file() { self.oeffne(&bank) }
            }
            Err(e) => { self.log.push(format!("{rel}: Umwandlung fehlgeschlagen: {e}")); self.schublade_zeigen() }
        }
    }
    /// Einzeltasten der Studios (Buchstaben, Ziffern, Satzzeichen ohne Cmd/Alt) sind per Einstellung abschaltbar; Pfeile, Leertaste, Esc, Entf und Cmd-/Alt-Kürzel gelten immer
    /// Takt-Anzeige der Statusleiste: Takt, Norm, Zeilen und Zyklen des Zielsystems (und im Debug-Kontext der Programmzähler); leer, wenn aus, kein Projektziel oder das Fenster zu schmal ist
    fn takt_anzeige(&self, pal: Pal) -> Element<'_, Msg> {
        if !self.einst.takt_anzeige || self.fenster.width / self.einst.zoom < 760.0 { return space::horizontal().width(0).into() }
        let (sys, norm) = match (&self.info, self.proj.pf.as_ref().and_then(|pf| pf.targets.first())) {
            (Some(i), t) => (i.target.clone(), t.map(|t| t.variant.to_ascii_lowercase()).unwrap_or_default()),
            (None, Some(t)) => (if t.system.is_empty() { t.id.clone() } else { t.system.clone() }, t.variant.to_ascii_lowercase()),
            _ => return space::horizontal().width(0).into() };
        let Some(mut t) = telemetrie::takt_text(&self.data, &sys, &norm) else { return space::horizontal().width(0).into() };
        if let Some(d) = &self.dbg { let s = d.snapshot(); if s.have_regs { t.push_str(&format!(" · PC ${:04X}", s.pc)) } }
        text(t).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.mute).into()
    }
    /// Geparstes Markdown (einmal je Inhalt)
    pub(crate) fn md_doc(&self, text: &str) -> std::rc::Rc<vc_md::Doc> {
        use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new(); text.hash(&mut h); let k = h.finish();
        let mut c = self.md_cache.borrow_mut(); if c.len() > 32 { c.clear() }
        c.entry(k).or_insert_with(|| std::rc::Rc::new(vc_md::parse(text))).clone()
    }
    /// Markdown-Datei als Vorschau zeigen? (Standard für `.md` mit Inhalt; „Quelltext“ schaltet auf den Editor um; leere Dateien öffnen im Editor)
    pub(crate) fn md_vorschau(&self, n: &str) -> bool { (n.to_lowercase().ends_with(".md") || self.md_als.contains(n)) && !self.md_quelltext.contains(n) && self.editoren.get(n).is_some_and(|e| !e.text().trim().is_empty()) }
    /// Dateibaum-Tasten: Pfeile, Pos1/Ende, →/←, Enter/Leertaste; `true`, wenn die Taste verbraucht wurde
    fn baum_taste(&mut self, k: &keyboard::Key) -> bool {
        use keyboard::key::Named; let keyboard::Key::Named(n) = k else { return false };
        if !matches!(n, Named::ArrowUp | Named::ArrowDown | Named::ArrowLeft | Named::ArrowRight | Named::Home | Named::End | Named::Enter | Named::Space) { return false }
        let zeilen = project::baum_zeilen(&self.proj, &self.baum_zu, &self.baum_filter, &self.exportiert); if zeilen.is_empty() { return false }
        let aktuell = self.baum_sel.as_ref().and_then(|s| zeilen.iter().position(|z| &z.rel == s)).or_else(|| self.name().and_then(|n| zeilen.iter().position(|z| z.rel == n)));
        let setze = |app: &mut App, i: usize| { app.baum_sel = Some(zeilen[i].rel.clone()) };
        let Some(i) = aktuell else { setze(self, 0); return true };
        match n {
            Named::ArrowDown => setze(self, (i + 1).min(zeilen.len() - 1)), Named::ArrowUp => setze(self, i.saturating_sub(1)),
            Named::Home => setze(self, 0), Named::End => setze(self, zeilen.len() - 1),
            Named::ArrowRight => { let z = &zeilen[i]; if z.ordner { if z.offen { if i + 1 < zeilen.len() && zeilen[i + 1].tiefe > z.tiefe { setze(self, i + 1) } } else { let rel = z.rel.clone(); self.baum_sel = Some(rel.clone()); let _ = self.update_innen(Msg::BaumKlapp(rel)); } } else { setze(self, i) } }
            Named::ArrowLeft => { let z = &zeilen[i]; if z.ordner && z.offen { let rel = z.rel.clone(); self.baum_sel = Some(rel.clone()); let _ = self.update_innen(Msg::BaumKlapp(rel)); } else if let Some(p) = (0..i).rev().find(|j| zeilen[*j].ordner && zeilen[*j].tiefe + 1 == z.tiefe) { setze(self, p) } }
            _ => { let z = &zeilen[i]; let rel = z.rel.clone(); self.baum_sel = Some(rel.clone()); if z.ordner { let _ = self.update_innen(Msg::BaumKlapp(rel)); } else { self.oeffne(&rel) } }
        }
        true
    }
    /// Layout für Hinweise und Klaviertasten-Beschriftung (Einstellung, sonst erkannt)
    /// Kam derselbe Befehl kurz zuvor auf dem anderen Weg (Menü ↔ Tastatur) an? Dann ist es dasselbe Kürzel, das macOS doppelt meldet (Menü-Kürzel und Tastenereignis)
    /// Eine Zeile der Kürzel-Diagnose ins Protokoll (höchstens die letzten 500 Diagnosezeilen bleiben)
    fn diag_zeile(&mut self, z: String) { self.log.push(z); let n = self.log.iter().filter(|l| l.starts_with("Kürzel: ")).count(); if n > 500 { if let Some(i) = self.log.iter().position(|l| l.starts_with("Kürzel: ")) { self.log.remove(i); } } }
    pub(crate) fn doppelt(&mut self, c: Cmd, menue: bool) -> bool {
        let jetzt = std::time::Instant::now();
        let d = self.doppel.is_some_and(|(vc, vm, t)| vc == c && vm != menue && jetzt.duration_since(t).as_millis() < self.doppel_ms);
        self.doppel = if d { None } else { Some((c, menue, jetzt)) }; d
    }
    pub(crate) fn layout(&self) -> einstellungen::Tastaturlayout { tastatur::aufloesen(self.einst.tastaturlayout, self.layout_gelernt, self.layout_system) }
    fn studio_taste_erlaubt(&self, k: &keyboard::Key, m: keyboard::Modifiers) -> bool { self.einst.studio_einzeltasten || !matches!(k, keyboard::Key::Character(_)) || m.command() || m.alt() || m.control() }
    /// Alle geöffneten Dateien beider Gruppen des Dokumentenfelds
    fn offene_dateien(&self) -> std::collections::BTreeSet<String> {
        let mut v: std::collections::BTreeSet<String> = self.tabs.iter().filter_map(|t| if let Tab::Datei(n) = t { Some(n.clone()) } else { None }).collect();
        if let Some((a, _)) = &self.andere { v.extend(a.iter().filter_map(|t| if let Tab::Datei(n) = t { Some(n.clone()) } else { None })) }
        v
    }
    /// Zugeklappte Ordner des Dateibaums für dieses Projekt merken
    fn baum_zustand_sichern(&self) { if let Some(f) = &self.baum_datei { let _ = vc_ide_model::treestate::save_to(f, &self.root, &self.baum_zu); } }
    /// Ordner, in dem „Neu …“ anlegt: der gewählte Ordner, bei einer Datei ihr Ordner, sonst das Projekt
    /// Offene Editoren, Tabs und Textmeta nach dem Verschieben einer Datei nachführen
    fn datei_verschoben(&mut self, alt: &str, neu: &str) {
        if let Some(mt) = self.textmeta.remove(alt) { self.textmeta.insert(neu.to_string(), mt); }
        if let Some(e) = self.editoren.remove(alt) { let lsp_offen = self.lsp_gesendet.remove(alt).is_some(); self.diagnosen.remove(alt);
            if lsp_offen { self.lsp.senden(lsp::Cmd::Close { uri: lsp::uri(&self.root, alt) }) }
            let mut e = e; e.lang = vc_ide_model::lang::Lang::of(neu);
            if neu.ends_with(".asm") || neu.ends_with(".inc") { self.lsp.senden(lsp::Cmd::Open { uri: lsp::uri(&self.root, neu), text: e.text() }); self.lsp_gesendet.insert(neu.to_string(), e.version); }
            self.editoren.insert(neu.to_string(), e); }
        for t in self.tabs.iter_mut().chain(self.andere.iter_mut().flat_map(|a| a.0.iter_mut())) { if *t == Tab::Datei(alt.to_string()) { *t = Tab::Datei(neu.to_string()) } }
    }
    /// Zeile des Dateibaums mit Ziehen: Drücken merkt die Quelle, Darüberfahren das Ziel, Loslassen führt aus (ohne Bewegung zählt es als Klick)
    fn zug_zeile<'a>(&self, inhalt: Element<'a, Msg>, rel: &str, ordner: bool) -> Element<'a, Msg> {
        iced::widget::mouse_area(inhalt).on_press(Msg::BaumZugStart(rel.to_string(), ordner)).on_release(Msg::BaumZugLos(rel.to_string())).on_enter(Msg::BaumZugUeber(rel.to_string())).into()
    }
    /// Datei oder Ordner im Baum in einen anderen Ordner verschieben; `.include`/`.incbin`-Pfade und Projektverweise ziehen mit, vorher wird gesichert (Rückgängig: Befehl „Verschieben rückgängig“)
    fn baum_verschieben(&mut self, quelle: &str, ziel_ordner: &str) {
        use vc_ide::reorganize as ro;
        let name = quelle.rsplit('/').next().unwrap_or(quelle).to_string();
        let neu = if ziel_ordner.is_empty() { name.clone() } else { format!("{ziel_ordner}/{name}") };
        if neu == quelle { return }
        if ziel_ordner == quelle || ziel_ordner.starts_with(&format!("{quelle}/")) { self.log.push("Ein Ordner lässt sich nicht in sich selbst verschieben.".into()); return }
        if self.editoren.values().any(|e| e.dirty) { self.log.push("Verschieben: erst alle geänderten Dateien speichern (Pfade in Quelltexten werden angepasst).".into()); return }
        let ist_ordner = project::pfad(&self.root, quelle).is_dir();
        let paare: Vec<(String, String)> = if ist_ordner { ro::list_files(&self.root).into_iter().filter_map(|f| f.strip_prefix(&format!("{quelle}/")).map(|r| (f.clone(), format!("{neu}/{r}")))).collect() } else { vec![(quelle.to_string(), neu.clone())] };
        if paare.is_empty() { self.log.push(format!("{quelle}: nichts zu verschieben")); return }
        let moves: Vec<ro::Move> = paare.iter().map(|(a, b)| ro::Move { from: a.clone(), to: b.clone(), category: vc_ide::importfiles::Category::Source, label: "", conflict: false }).collect();
        let (Some(file), Some(mut pf)) = (self.proj.file.clone(), self.proj.pf.clone()) else { self.log.push("Verschieben braucht eine Projektdatei (.vcprj).".into()); return };
        match ro::apply(&self.root, &self.root.join(&file), &mut pf, &moves, None, true, &|_| {}) {
            Ok(rep) => {
                for (a, b) in &paare { self.datei_verschoben(a, b) }
                let wohin = |f: &str| paare.iter().find(|(a, _)| a == f).map_or(f.to_string(), |(_, b)| b.clone());
                for (f, _) in &rep.rewritten { let f = wohin(f); if self.editoren.contains_key(&f) { self.editor_neu_lesen(&f) } }
                if ist_ordner { loesche_leere_ordner(&self.root.join(quelle)) }
                self.projekt_neu_laden(); self.baum_zu.remove(ziel_ordner);
                let angepasst: usize = rep.rewritten.iter().map(|(_, n)| n).sum();
                self.log.push(format!("Verschoben: {quelle} → {neu}{}", if angepasst > 0 { format!(" ({angepasst} Pfad(e) in Quelltexten angepasst)") } else { String::new() })); self.markiere()
            }
            Err(e) => self.log.push(format!("Verschieben fehlgeschlagen: {e}")),
        }
    }
    /// Verschieben im Baum zurücknehmen (die letzte Sicherung unter `.vcbackup/`); nichts wird überschrieben, wenn sich Dateien seitdem geändert haben
    fn baum_rueckgaengig(&mut self) {
        use vc_ide::reorganize as ro;
        let Some(l) = ro::letzte(&self.root) else { self.log.push("Es gibt nichts rückgängig zu machen.".into()); return };
        if self.editoren.values().any(|e| e.dirty) { self.log.push("Rückgängig: erst alle geänderten Dateien speichern.".into()); return }
        match ro::undo_geprueft(&self.root, &l) {
            Ok(n) => {
                let _ = std::fs::remove_dir_all(&l.backup); // verbraucht: der nächste Aufruf nimmt den Schritt davor zurück
                for (a, b) in &l.moves { self.datei_verschoben(b, a) }
                let offen: Vec<String> = self.editoren.keys().cloned().collect(); for f in offen { if f.ends_with(".asm") || f.ends_with(".inc") || f.ends_with(".bas") { self.editor_neu_lesen(&f) } }
                let alte: std::collections::BTreeSet<String> = l.moves.iter().filter_map(|(_, b)| b.rsplit_once('/').map(|(o, _)| o.to_string())).collect(); for o in alte { loesche_leere_ordner(&self.root.join(o)) }
                self.projekt_neu_laden(); self.log.push(format!("Verschieben rückgängig gemacht ({n} Datei(en))")); self.markiere()
            }
            Err(e) => self.log.push(e),
        }
    }
    /// Editor einer geöffneten Datei neu von der Platte lesen (nach Änderungen durch Verschieben)
    fn editor_neu_lesen(&mut self, rel: &str) {
        let Ok((t, m)) = textdatei::lesen(&project::pfad(&self.root, rel)) else { return };
        let (alt_cursor, wrap) = self.editoren.get(rel).map_or((None, self.einst.umbruch), |e| (Some(e.cursor), e.wrap));
        self.textmeta.insert(rel.into(), m);
        let mut e = Editor::new(&t); e.lang = vc_ide_model::lang::Lang::of(rel); e.wrap = wrap; e.dirty = false;
        if let Some(c) = alt_cursor { let line = c.line.min(e.lines.len() - 1); let col = c.col.min(e.line_len(line)); e.set_cursor(Pos { line, col }, false) }
        if rel.ends_with(".asm") || rel.ends_with(".inc") { self.lsp.senden(lsp::Cmd::Open { uri: lsp::uri(&self.root, rel), text: e.text() }); self.lsp_gesendet.insert(rel.into(), e.version); }
        self.editoren.insert(rel.into(), e);
    }
    fn ordner_zu(&self, rel: String) -> String { if rel.is_empty() || project::pfad(&self.root, &rel).is_dir() { rel } else { rel.rsplit_once('/').map_or(String::new(), |(o, _)| o.to_string()) } }
    fn oeffne(&mut self, rel: &str) {
        // Bilder, Binärdaten und Archive öffnen nicht als Text, sondern im Standardprogramm des Systems
        { let pf = project::pfad(&self.root, rel); if pf.is_file() && !self.editoren.contains_key(rel) && !vc_ide::diskimage::is_image_name(rel) && !(rel.ends_with(".vcsprite") || rel.ends_with(".vcpmg") || rel.ends_with(".vcchar") || rel.ends_with(".vcsong") || rel.ends_with(".vcplay") || rel.ends_with(".vcscreen") || rel.ends_with(".vctiles") || rel.ends_with(".vcpal") || rel.ends_with(".asset.json")) && project::ist_nichttext(&pf) && !hexansicht::hex_statt_extern(rel) { self.log.push(format!("{rel}: keine Textdatei, öffnet im Standardprogramm")); aussen::ausfuehren(aussen::Aufruf::Standardprogramm(pf)); return } }
        if self.hexe.contains_key(rel) && !project::ist_nichttext(&project::pfad(&self.root, rel)) { self.hexe.remove(rel); }
        if (!self.editoren.contains_key(rel) && project::pfad(&self.root, rel).is_file() && project::ist_nichttext(&project::pfad(&self.root, rel)) && !rel.ends_with(".vcsprite") && !rel.ends_with(".vcpmg") && !rel.ends_with(".vcchar") && !rel.ends_with(".vcsong") && !rel.ends_with(".vcplay") && !rel.ends_with(".vcscreen") && !rel.ends_with(".vctiles") && !rel.ends_with(".vcpal") && !rel.ends_with(".asset.json") && !vc_ide::diskimage::is_image_name(rel)) { self.hex_oeffnen(rel); return }
        // Medien mit eigenem Studio (Sprite, Player/Missile, Zeichensatz) öffnen im Studio statt im Texteditor
        if self.studios.contains_key(rel) || ((rel.ends_with(".vcsprite") || rel.ends_with(".vcpmg") || rel.ends_with(".vcchar") || rel.ends_with(".asset.json")) && studio::ist_pixel_medium(&project::pfad(&self.root, rel))) { self.studio_oeffnen(rel); return }
        if self.disks.contains_key(rel) || vc_ide::diskimage::is_image_name(rel) { self.disk_oeffnen(rel); return }
        if self.songs.contains_key(rel) || ((rel.ends_with(".vcsong") || rel.ends_with(".vcplay")) && std::fs::read_to_string(project::pfad(&self.root, rel)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).is_some_and(|v| v["kind"] == "song" || v["kind"] == "play")) { self.song_oeffnen(rel); return }
        if self.banken.contains_key(rel) || (rel.ends_with(".vcscreen") && std::fs::read_to_string(project::pfad(&self.root, rel)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).is_some_and(|v| vc_media::tiles::is_bank(&v))) { self.bank_oeffnen(rel); return }
        if self.tilesets.contains_key(rel) || (rel.ends_with(".vctiles") && std::fs::read_to_string(project::pfad(&self.root, rel)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).is_some_and(|v| v["kind"] == "tileset")) { self.tile_oeffnen(rel); return }
        if self.screens.contains_key(rel) || (rel.ends_with(".vcscreen") && std::fs::read_to_string(project::pfad(&self.root, rel)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).is_some_and(|v| vc_ide_model::screendoc::ist_textbildschirm(&v))) { self.screen_oeffnen(rel); return }
        if self.paletten.contains_key(rel) || ((rel.ends_with(".vcpal") || rel.ends_with(".asset.json")) && palettestudio::medienart(&project::pfad(&self.root, rel)).as_deref() == Some("palette")) { self.palette_oeffnen(rel); return }
        if rel.ends_with(".vctiles") && !self.editoren.contains_key(rel) && std::fs::read_to_string(project::pfad(&self.root, rel)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).is_some_and(|v| v["kind"] == "tilemap") {
            self.rueckfrage = Some(Rueckfrage::Kachelkarte(rel.to_string())); return
        }
        if !self.editoren.contains_key(rel) {
            let gelesen = textdatei::lesen(&project::pfad(&self.root, rel)).or_else(|_| textdatei::lesen(std::path::Path::new(rel)));
            let txt = match gelesen { Ok((t, m)) => { self.textmeta.insert(rel.into(), m); t } Err(e) => { self.lesefehler.insert(rel.into()); self.log.push(format!("{e} (wird nicht überschrieben)")); format!("; {rel}: nicht lesbar") } };
            let mut e = Editor::new(&txt); e.lang = vc_ide_model::lang::Lang::of(rel); e.wrap = self.einst.umbruch; e.dirty = false;
            if let Some((l, c)) = self.marken.get(rel).copied() { let line = l.saturating_sub(1).min(e.lines.len() - 1); let col = c.saturating_sub(1).min(e.line_len(line)); e.set_cursor(Pos { line, col }, false); e.center_on(line) }
            self.editoren.insert(rel.into(), e);
        }
        if rel.ends_with(".asm") || rel.ends_with(".inc") { let e = &self.editoren[rel]; self.lsp.senden(lsp::Cmd::Open { uri: lsp::uri(&self.root, rel), text: e.text() }); self.lsp_gesendet.insert(rel.into(), e.version); }
        if rel.ends_with(".asm") || rel.ends_with(".inc") { self.letzte_asm = Some(rel.to_string()) }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.markiere(); self.editor_fokus();
    }
    /// Cursorstellen aller offenen Dateien merken (beim Öffnen steht der Cursor wieder dort, wo er war)
    fn wort_unter_cursor(&self) -> Option<String> {
        let e = self.name().and_then(|n| self.editoren.get(&n))?; let l: Vec<char> = e.lines[e.cursor.line].chars().collect(); let c = e.cursor.col.min(l.len());
        let wortzeichen = |x: char| x.is_alphanumeric() || x == '_' || x == '.' || x == '$';
        let (mut a, mut b) = (c, c); while a > 0 && wortzeichen(l[a - 1]) { a -= 1 } while b < l.len() && wortzeichen(l[b]) { b += 1 }
        (b > a).then(|| l[a..b].iter().collect())
    }
    /// Zeilen des offenen Kontextmenüs mit den zugehörigen Befehlen; ausgegraut, was jetzt nicht geht
    fn kontext_zeilen(&self, k: &Kontext) -> (Vec<Zeile<Msg>>, Vec<Option<Cmd>>) {
        let mac = cfg!(target_os = "macos") && !self.kuerzel_windows; let mut zeilen = vec![]; let mut cmds = vec![];
        if k.art == Art::Menue {
            // Ziel: „*“ (alle Menüs untereinander), „Menü“ oder „Menü/Untermenü“
            let ziel = k.ziel.as_deref().unwrap_or(""); let alle = ziel == "*"; let (haupt, unter) = ziel.split_once('/').map_or((ziel, ""), |(a, b)| (a, b));
            let eintrag = |i: &befehle::Info, zeilen: &mut Vec<Zeile<Msg>>, cmds: &mut Vec<Option<Cmd>>| { let ok = self.nutzbar(i.cmd); let n = befehle::menuename(i); zeilen.push(Zeile { name: if ok { n.to_string() } else { format!("{} ({})", n, Self::menue_grund(i.cmd)) }, kuerzel: kontext::kuerzel(i.cmd, mac), aktiv: ok, msg: Some(Msg::KontextWahl(i.cmd)) }); cmds.push(Some(i.cmd)); };
            for m in befehle::MENUES.iter().filter(|m| alle || **m == haupt) {
                if alle { zeilen.push(Zeile { name: m.to_uppercase(), kuerzel: String::new(), aktiv: false, msg: None }); cmds.push(None) }
                if !alle && !unter.is_empty() {
                    zeilen.push(Zeile { name: format!("◂ {m}"), kuerzel: String::new(), aktiv: true, msg: Some(Msg::MenueAuf(m.to_string())) }); cmds.push(None);
                    zeilen.push(Zeile { name: unter.to_uppercase(), kuerzel: String::new(), aktiv: false, msg: None }); cmds.push(None);
                    for i in befehle::menue_eintraege(m, unter) { eintrag(i, &mut zeilen, &mut cmds) }
                    continue;
                }
                for i in befehle::menue_eintraege(m, "") { eintrag(i, &mut zeilen, &mut cmds) }
                for u in befehle::untermenues(m) {
                    if alle { zeilen.push(Zeile { name: u.to_string(), kuerzel: String::new(), aktiv: false, msg: None }); cmds.push(None); for i in befehle::menue_eintraege(m, u) { eintrag(i, &mut zeilen, &mut cmds) } }
                    else { if befehle::untermenues(m).first() == Some(u) && !zeilen.is_empty() { zeilen.push(Zeile { name: String::new(), kuerzel: String::new(), aktiv: false, msg: None }); cmds.push(None) } zeilen.push(Zeile { name: format!("{u} ▸"), kuerzel: String::new(), aktiv: true, msg: Some(Msg::MenueAuf(format!("{m}/{u}"))) }); cmds.push(None) }
                }
            }
            return (zeilen, cmds);
        }
        if k.art == Art::FlaecheStudio {
            let mut gruppe = None;
            for x in self.studio_aktionen().into_iter().filter(|x| x.menue) {
                if gruppe.is_some_and(|g| g != x.gruppe) { zeilen.push(Zeile { name: String::new(), kuerzel: String::new(), aktiv: false, msg: None }); cmds.push(None) }
                gruppe = Some(x.gruppe); zeilen.push(Zeile { name: x.name, kuerzel: befehle::plattform_text(&x.kuerzel), aktiv: x.aktiv, msg: Some(Msg::KontextMsg(Box::new(x.msg))) }); cmds.push(None);
            }
            return (zeilen, cmds);
        }
        let (sel, n_tabs) = (self.name().and_then(|n| self.editoren.get(&n)).map_or(false, |e| e.selection().is_some()), self.tabs.len());
        for e in kontext::eintraege(k.art) {
            match e {
                None => { zeilen.push(Zeile { name: String::new(), kuerzel: String::new(), aktiv: false, msg: None }); cmds.push(None) }
                Some(c) => {
                    let ok = self.nutzbar(*c) && match c {
                        Cmd::Ausschneiden | Cmd::Kopieren if k.art == Art::Flaeche && self.name().is_some_and(|n| self.songs.contains_key(&n)) => true,
                        Cmd::Einfuegen if k.art == Art::Flaeche && self.name().is_some_and(|n| self.songs.contains_key(&n)) => self.name().and_then(|n| self.songs.get(&n)).is_some_and(|s| s.doc.zwischen.is_some()),
                        Cmd::Rueckgaengig if k.art == Art::Flaeche && self.name().is_some_and(|n| self.songs.contains_key(&n)) => self.name().and_then(|n| self.songs.get(&n)).is_some_and(|s| s.doc.kann_undo()),
                        Cmd::Wiederholen if k.art == Art::Flaeche && self.name().is_some_and(|n| self.songs.contains_key(&n)) => self.name().and_then(|n| self.songs.get(&n)).is_some_and(|s| s.doc.kann_redo()),
                        Cmd::Ausschneiden | Cmd::Kopieren if k.art == Art::Flaeche => self.name().and_then(|n| self.studios.get(&n)).is_some_and(|s| s.doc.sel.is_some()),
                        Cmd::Einfuegen if k.art == Art::Flaeche => self.name().and_then(|n| self.studios.get(&n)).is_some_and(|s| s.doc.clip.is_some()),
                        Cmd::Rueckgaengig if k.art == Art::Flaeche => self.name().and_then(|n| self.studios.get(&n)).is_some_and(|s| s.doc.kann_undo()),
                        Cmd::Wiederholen if k.art == Art::Flaeche => self.name().and_then(|n| self.studios.get(&n)).is_some_and(|s| s.doc.kann_redo()),
                        Cmd::Ausschneiden | Cmd::Kopieren => sel,
                        Cmd::ReiterLinks => self.aktiv > 0, Cmd::ReiterRechts => self.aktiv + 1 < n_tabs,
                        Cmd::ReiterAndereHaelfte | Cmd::Teilen => n_tabs >= 2 || self.andere.is_some(),
                        _ => true };
                    zeilen.push(Zeile { name: kontext::eintragsname(*c).to_string(), kuerzel: kontext::kuerzel(*c, mac), aktiv: ok, msg: Some(Msg::KontextWahl(*c)) }); cmds.push(Some(*c));
                }
            }
        }
        (zeilen, cmds)
    }
    /// Menüleiste im Fenster: unter macOS gibt es die Menüleiste des Systems (`native.rs`); sonst nach Einstellung. Im Zen-Modus ausgeblendet.
    pub(crate) fn menue_sichtbar(&self) -> bool { !self.layout.zen() && (self.menue_test || (!cfg!(target_os = "macos") && self.einst.menueleiste)) }
    /// Buchstaben, die mit Alt ein Menü öffnen, solange die Menüleiste im Fenster sichtbar ist (sonst leer: dann bleibt Alt+Taste ein Textzeichen)
    /// Alt+Buchstabe als Menü-Kürzel gibt es nur unter Windows und Linux; unter macOS bleibt Option+Taste immer ein Textzeichen (`kuerzel_windows` schaltet für Tests und Bilder auf die Windows/Linux-Pfade)
    fn alt_mnemonics(&self) -> bool { self.menue_sichtbar() && (!cfg!(target_os = "macos") || self.kuerzel_windows) }
    fn alt_menue_buchstaben(&self) -> Vec<char> { if self.alt_mnemonics() { befehle::MENUES.iter().filter_map(|m| menue_buchstabe(m)).collect() } else { vec![] } }
    /// Zu schmal für alle Menütitel: ein Knopf „Menü“ mit allen Menüs untereinander
    fn menue_schmal(&self) -> bool { self.fenster.width / self.einst.zoom < 560.0 }
    /// Was ein Befehl braucht, wenn er gerade ausgegraut ist (aus der Befehlstabelle)
    fn menue_grund(c: Cmd) -> &'static str {
        match c {
            Cmd::DebugHierher => return "braucht eine Debug-Sitzung und einen offenen Quelltext",
            Cmd::DebugWeiter | Cmd::DebugPause | Cmd::DebugUeber | Cmd::DebugHinein | Cmd::DebugRaus | Cmd::DebugEnde => return "braucht eine Debug-Sitzung",
            Cmd::Exportieren => return "braucht ein geöffnetes Studio",
            Cmd::MdFett | Cmd::MdKursiv | Cmd::MdCode | Cmd::MdDurchgestrichen | Cmd::MdLink | Cmd::MdBild | Cmd::MdUeberschriftTiefer | Cmd::MdUeberschriftHoeher | Cmd::MdAufzaehlung | Cmd::MdNummerierung | Cmd::MdAufgaben | Cmd::MdZitat | Cmd::MdAufgabeUmschalten | Cmd::MdTabelleNeu | Cmd::MdTabelleAusrichten => return "braucht eine Markdown-Datei",
            _ => {}
        }
        match befehle::info(c).braucht { befehle::Braucht::Projekt => "braucht ein Projekt", befehle::Braucht::Editor => "braucht einen offenen Quelltext", befehle::Braucht::Nichts => "gerade nicht möglich" } }
    fn menue_oeffnen(&mut self, name: &str) {
        let idx = befehle::MENUES.iter().position(|m| *m == name.split('/').next().unwrap_or("")).unwrap_or(0);
        let pos = if name == "*" { iced::Point::new(8.0, 26.0) } else { iced::Point::new(8.0 + idx as f32 * 86.0, 26.0) };
        self.kontext = Some(Kontext { art: Art::Menue, pos, ziel: Some(name.to_string()), sel: 0 }); self.kontext_sel_setzen();
    }
    /// Hält die per Tastatur gewählte Zeile im sichtbaren Teil eines langen Menüs der Menüleiste
    fn menue_scrollen(&self) -> Task<Msg> {
        let Some(k) = self.kontext.as_ref().filter(|k| k.art == Art::Menue) else { return Task::none() };
        let (z, _) = self.kontext_zeilen(k); if z.len() < 2 { return Task::none() }
        iced::widget::operation::snap_to(iced::widget::Id::new("menue"), iced::widget::scrollable::RelativeOffset { x: 0.0, y: k.sel as f32 / (z.len() - 1) as f32 })
    }
    /// Nächstes oder voriges Menü (Pfeiltasten links/rechts bei offenem Menü der Menüleiste)
    fn menue_weiter(&mut self, d: i32) {
        let Some(k) = &self.kontext else { return }; if k.art != Art::Menue { return }
        let n = befehle::MENUES.len() as i32; let i = befehle::MENUES.iter().position(|m| Some(*m) == k.ziel.as_deref().and_then(|z| z.split('/').next())).map_or(0, |i| i as i32);
        let neu = befehle::MENUES[(i + d).rem_euclid(n) as usize]; self.menue_oeffnen(neu);
    }
    /// Die Menüleiste: eine schmale Zeile über der Kopfzeile mit den Menüs aus der Befehlstabelle
    fn menueleiste(&self, pal: Pal) -> Element<'_, Msg> {
        let offen = self.kontext.as_ref().filter(|k| k.art == Art::Menue).and_then(|k| k.ziel.clone()); let mnemonik = self.alt_mnemonics();
        let titel = |name: String, anz: String, breit: f32| -> Element<'_, Msg> {
            let an = offen.as_deref().and_then(|o| o.split('/').next()) == Some(name.as_str());
            // Mnemonic: der Buchstabe für Alt+Buchstabe ist unterstrichen
            let inhalt: Element<'_, Msg> = match menue_buchstabe(&name).filter(|_| mnemonik).and_then(|b| anz.to_lowercase().find(b).map(|i| (i, b))) {
                Some((i, b)) => { let n = b.len_utf8(); let sp: Vec<iced::widget::text::Span<'static, (), iced::Font>> = vec![span(anz[..i].to_string()).color(pal.text), span(anz[i..i + n].to_string()).color(pal.text).underline(true), span(anz[i + n..].to_string()).color(pal.text)]; container(rich_text(sp).size(12.5)).center_x(Length::Fill).into() }
                None => text(anz).size(12.5).color(pal.text).center().into() };
            button(inhalt).width(Length::Fixed(breit)).padding([3, 6]).on_press(Msg::MenueAuf(name)).style(move |_, st| button::Style { background: (an || matches!(st, button::Status::Hovered | button::Status::Pressed)).then(|| pal.sel.into()), text_color: pal.text, border: Border { radius: 4.0.into(), ..Border::default() }, ..Default::default() }).into()
        };
        let mut r = row![].spacing(0).align_y(Alignment::Center);
        if self.menue_schmal() { r = r.push(titel("*".into(), "☰ Menü".into(), 86.0)) }
        else { for (i, m) in befehle::MENUES.iter().enumerate() { let anz = format!("{m}"); let _ = i; r = r.push(titel(m.to_string(), anz, 86.0)) } }
        container(r.push(space::horizontal())).padding([2, 8]).width(Length::Fill).height(26).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 0.0.into() }, ..Default::default() }).into()
    }
    fn kontext_sel_setzen(&mut self) { let Some(k) = &self.kontext else { return }; let (z, _) = self.kontext_zeilen(k); let sel = kontext::naechste(&z, z.len().saturating_sub(1), 1); if let Some(k) = &mut self.kontext { k.sel = sel } }
    fn kontext_pfeil(&mut self, d: i32) { let Some(k) = &self.kontext else { return }; let (z, _) = self.kontext_zeilen(k); let sel = kontext::naechste(&z, k.sel, d); if let Some(k) = &mut self.kontext { k.sel = sel } }
    fn kontext_befehl_unter_auswahl(&self) -> Option<Cmd> { let k = self.kontext.as_ref()?; let (z, c) = self.kontext_zeilen(k); if z.get(k.sel).is_some_and(|z| z.aktiv) { c.get(k.sel).copied().flatten() } else { None } }
    fn marken_sichern(&mut self) {
        for (rel, e) in &self.editoren { self.marken.insert(rel.clone(), (e.cursor.line + 1, e.cursor.col + 1)); }
        if let Some(f) = &self.marken_datei { let _ = vc_ide_model::cursors::save_to(f, &self.root, &self.marken); }
    }
    fn name(&self) -> Option<String> { match self.tabs.get(self.aktiv) { Some(Tab::Datei(n)) => Some(n.clone()), _ => None } }
    fn ed(&mut self) -> Option<&mut Editor> { let n = self.name()?; self.editoren.get_mut(&n) }
    fn markiere(&mut self) {
        let Some(n) = self.name() else { return };
        let f: HashMap<usize, String> = self.probleme.iter().filter(|p| p.datei == n && p.fehler).map(|p| (p.zeile.saturating_sub(1), p.text.clone())).collect();
        let w: HashMap<usize, String> = self.probleme.iter().filter(|p| p.datei == n && !p.fehler).map(|p| (p.zeile.saturating_sub(1), p.text.clone())).collect();
        let (mut f, mut w) = (f, w);
        let mut h: HashMap<usize, String> = HashMap::new();
        for d in self.diagnosen.get(&n).into_iter().flatten() { if d.fehler { f.entry(d.zeile).or_insert_with(|| d.text.clone()); } else if d.hinweis { h.entry(d.zeile).or_insert_with(|| d.text.clone()); } else { w.entry(d.zeile).or_insert_with(|| d.text.clone()); } }
        if let Some(e) = self.editoren.get_mut(&n) { e.errors = f; e.warns = w; e.hints = h }
    }
    /// Ist der Befehl jetzt nutzbar? (Konzept Abschnitt 7: sonst ausgegraut)
    fn nutzbar(&self, c: Cmd) -> bool { if matches!(c, Cmd::MdFett | Cmd::MdKursiv | Cmd::MdCode | Cmd::MdDurchgestrichen | Cmd::MdLink | Cmd::MdBild | Cmd::MdUeberschriftTiefer | Cmd::MdUeberschriftHoeher | Cmd::MdAufzaehlung | Cmd::MdNummerierung | Cmd::MdAufgaben | Cmd::MdZitat | Cmd::MdAufgabeUmschalten | Cmd::MdTabelleNeu | Cmd::MdTabelleAusrichten) { return self.md_aktiv() } if c == Cmd::MdAlsMarkdown { return self.name().is_some_and(|n| self.editoren.contains_key(&n)) } if c == Cmd::Exportieren { return self.name().is_some_and(|n| self.studios.contains_key(&n) || self.paletten.contains_key(&n) || self.screens.contains_key(&n) || self.tilesets.contains_key(&n) || self.banken.contains_key(&n) || self.songs.contains_key(&n)) } if matches!(c, Cmd::DebugWeiter | Cmd::DebugPause | Cmd::DebugUeber | Cmd::DebugHinein | Cmd::DebugRaus | Cmd::DebugEnde | Cmd::DebugHierher) { return self.dbg.is_some() && (c != Cmd::DebugHierher || self.name().is_some()) } match befehle::info(c).braucht { Braucht::Nichts => true, Braucht::Projekt => true, Braucht::Editor => self.name().is_some() } }
    #[cfg(target_os = "macos")]
    fn a11y(&self) {
        let reiter = self.tabs.iter().enumerate().map(|(i, x)| match x { Tab::Datei(n) => (n.clone(), i == self.aktiv, (self.editoren.get(n).map_or(false, |e| e.dirty) || self.studios.get(n).map_or(false, |s| s.doc.dirty()) || self.paletten.get(n).map_or(false, |p| p.dirty()) || self.disks.get(n).map_or(false, |d| d.dirty) || self.screens.get(n).map_or(false, |s| s.doc.dirty()) || self.tilesets.get(n).map_or(false, |s| s.doc.dirty()) || self.banken.get(n).map_or(false, |s| s.doc.dirty()) || self.songs.get(n).map_or(false, |s| s.doc.dirty()))), Tab::Einstellungen => (t::EINSTELLUNGEN.to_string(), i == self.aktiv, false), Tab::Start => ("Start".to_string(), i == self.aktiv, false), Tab::Projekt => ("Projekt".to_string(), i == self.aktiv, self.pe.as_ref().is_some_and(|z| z.geaendert())), Tab::Hilfe(k) => (self.hilfe_titel(k), i == self.aktiv, false) }).collect();
        let n = self.name(); let ed = n.as_ref().and_then(|n| self.editoren.get(n));
        native::a11y_update(&native::A11y { studio: self.studio_a11y(), reiter, dateien: project::baum_zeilen(&self.proj, &self.baum_zu, &self.baum_filter, &self.exportiert).into_iter().map(|z| { let sel = self.baum_sel.as_deref() == Some(z.rel.as_str()) || (self.baum_sel.is_none() && Some(&z.rel) == n.as_ref()); (if z.ordner { format!("{}, Ordner, {}, Ebene {}", z.name, if z.offen { "aufgeklappt" } else { "zugeklappt" }, z.tiefe + 1) } else { format!("{}, Datei, Ebene {}", z.name, z.tiefe + 1) }, sel) }).collect(), editor_name: n.map(|n| format!("Editor {n}")).unwrap_or_else(|| "Kein Dokument".into()), editor_wert: ed.map(|e| format!("Zeile {}: {}", e.cursor.line + 1, e.lines[e.cursor.line])).unwrap_or_default(), protokoll: self.log.last().cloned().unwrap_or_default(), baut: self.baut, streifen: self.streifen_zustand(), bereiche: self.bereiche_zustand(), kontext: self.kontext.as_ref().map(|k| (kontext::menuename(k.art).to_string(), self.kontext_zeilen(k).0.into_iter().filter(|z| !z.name.is_empty()).map(|z| (z.name, z.aktiv)).collect())) });
        native::aktiviere(|c| self.nutzbar(c));
    }
    #[cfg(not(target_os = "macos"))]
    fn a11y(&self) {}

    fn update(&mut self, m: Msg) -> Task<Msg> {
        match &m {
            Msg::Ed(_) | Msg::EdIn(..) | Msg::ReiterIn(..) | Msg::TabDruck(..) | Msg::Reiter(_) => self.fokus_bereich = Bereich::Dokument,
            Msg::SeiteWahl(_) | Msg::OeffneDatei(_) | Msg::BaumKlapp(_) | Msg::BaumFilter(_) | Msg::BaumNeuLaden | Msg::BaumAlleZu | Msg::SucheFrage(_) | Msg::SpalteAuf(_) | Msg::BaumNeu(..) | Msg::BaumOrdner(_) | Msg::BaumArtWahl(_) | Msg::BaumOrdnerWahl(_) | Msg::BaumZugStart(..) | Msg::BaumZugLos(_) | Msg::BaumProfil(_) | Msg::BaumUmbenennen | Msg::BaumLoeschen => self.fokus_bereich = Bereich::Seitenleiste,
            Msg::Schublade(_) | Msg::GotoProblem(_) => self.fokus_bereich = Bereich::Schublade,
            Msg::PanelWahl(_) | Msg::StreifenKlick(_) | Msg::HilfeSuche(_) | Msg::HilfeOeffne(_) | Msg::ZeichenSuche(_) | Msg::ZeichenEinfuegen(_) => self.fokus_bereich = Bereich::Panel,
            Msg::BereichAkt(b, _) | Msg::BereichAuf(b) => self.fokus_bereich = *b,
            _ => {}
        }
        let nachfuehren = matches!(m, Msg::SongAkt(..) | Msg::Taste(..) | Msg::TasteP(..) | Msg::SongMaus(..)); let r = self.update_innen(m); let r = if nachfuehren { Task::batch([r, self.song_nachfuehren()]) } else { r }; self.lsp_abgleich(); self.vorschlaege_anfragen(); self.a11y(); r
    }
    /// Geänderte Quelltexte an den Sprachdienst melden (er fasst Änderungen zusammen)
    fn lsp_abgleich(&mut self) {
        let Some(n) = self.name() else { return }; let Some(e) = self.editoren.get(&n) else { return };
        if self.lsp_gesendet.get(&n).is_some_and(|v| *v != e.version) { let v = e.version; self.lsp.senden(lsp::Cmd::Change { uri: lsp::uri(&self.root, &n), text: e.text(), version: v as i64 }); self.lsp_gesendet.insert(n, v); }
    }
    /// Nach dem Tippen Vorschläge vom Sprachdienst holen (nach der Änderung, damit er den neuen Text kennt)
    fn vorschlaege_anfragen(&mut self) {
        if !self.vorschlag_noetig { return } self.vorschlag_noetig = false;
        let Some(n) = self.name() else { return }; if !self.lsp_gesendet.contains_key(&n) { return }
        let Some(e) = self.editoren.get(&n) else { return }; let (v, p) = (e.version, e.cursor);
        self.vorschlag_req += 1; self.vorschlag_fuer = (v, p);
        self.lsp.senden(lsp::Cmd::Complete { uri: lsp::uri(&self.root, &n), req: self.vorschlag_req });
    }
    fn lsp_antwort(&mut self, a: lsp::Antwort) {
        match a {
            lsp::Antwort::Diagnose { uri, diag } => { let rel = self.lsp_gesendet.keys().find(|r| lsp::uri(&self.root, r) == uri).cloned(); if let Some(r) = rel { self.diagnosen.insert(r, diag); self.markiere() } }
            lsp::Antwort::Vorschlaege { req, uri, items } => {
                let Some(rel) = self.lsp_gesendet.keys().find(|r| lsp::uri(&self.root, r) == uri).cloned() else { return };
                if req != self.vorschlag_req { return }
                let (v, p) = self.vorschlag_fuer; let Some(e) = self.editoren.get_mut(&rel) else { return };
                e.lsp_items = items.into_iter().map(|i| (i.text, i.detail)).collect();
                if e.version == v && e.cursor == p { let vok: Vec<&str> = vc_gui_editor::highlight::MNEMONICS.iter().chain(vc_gui_editor::highlight::DIRECTIVES.iter()).copied().collect(); e.update_completion(&vok) }
            }
            lsp::Antwort::Definition { req, ziel } => {
                if req != self.def_req { return } let Some((uri, zeile)) = ziel else { self.log.push("Keine Definition gefunden.".into()); return };
                let rel = self.namen.iter().find(|r| lsp::uri(&self.root, r) == uri).cloned();
                match rel { Some(r) => { self.oeffne(&r); if let Some(e) = self.editoren.get_mut(&r) { let l = zeile.min(e.lines.len() - 1); e.set_cursor(Pos { line: l, col: 0 }, false); e.center_on(l) } } None => self.log.push(format!("Definition außerhalb des Projekts: {uri}")) }
            }
            lsp::Antwort::Hover { req, uri, zeile, spalte, text, bereich } => {
                let Some(rel) = self.lsp_gesendet.keys().find(|r| lsp::uri(&self.root, r) == uri).cloned() else { return };
                if req != self.hover_req { return }
                if let Some(e) = self.editoren.get_mut(&rel) {
                    let (s, en) = bereich.unwrap_or((spalte, spalte + 1)); let (von, bis) = (Pos { line: zeile, col: s }, Pos { line: zeile, col: en.max(s + 1) });
                    // gilt eine Fehlermeldung derselben Zeile, steht sie über dem Hinweis des Sprachdienstes
                    let diag = e.hover.as_ref().filter(|h| h.von.line == zeile && h.von.col == 0 && (h.text.starts_with("**Fehler**") || h.text.starts_with("**Warnung**"))).map(|h| h.text.clone());
                    e.hover = match (diag, text) { (Some(d), Some(t)) => Some(vc_gui_editor::core::HoverInfo { von: Pos { line: zeile, col: 0 }, bis: Pos { line: zeile, col: e.lines[zeile.min(e.lines.len() - 1)].chars().count().max(bis.col) }, text: format!("{d}\n\n{t}") }), (Some(d), None) => e.hover.take().map(|h| vc_gui_editor::core::HoverInfo { text: d, ..h }), (None, Some(t)) => Some(vc_gui_editor::core::HoverInfo { von, bis, text: t }), (None, None) => None };
                }
            }
        }
    }
    fn update_innen(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Start => { if !cfg!(test) && self.einst.splash { self.splash = true } let mut ts = vec![iced::system::theme().map(Msg::SystemSchema), self.update_start()]; ts.push(iced::window::oldest().and_then(iced::window::size).map(Msg::Fenster)); if !cfg!(test) { ts.push(iced::window::oldest().and_then(|id| iced::window::monitor_size(id).map(move |m| (id, m))).map(|(id, m)| Msg::Bildschirm(id, m))) } #[cfg(target_os = "macos")] { native::init(); ts.push(native::a11y_task()) } return Task::batch(ts) }
            Msg::A11yBereit | Msg::Nichts => {}
            Msg::SystemSchema(m) => self.sys_dunkel = !matches!(m, iced::theme::Mode::Light),
            Msg::Ed(e) => self.ed_nachricht(e),
            Msg::Menue(id) => { if self.einst.kuerzel_diagnose { self.diag_zeile(format!("Kürzel: Menü-Auslösung id={id}")) } if let Some(c) = befehle::nach_id(&id) { if self.doppelt(c, true) { if self.einst.kuerzel_diagnose { self.diag_zeile(format!("Kürzel: {id} als Doppelereignis verworfen")) } return Task::none() } return self.befehl(c) } }
            Msg::Befehl(c) => return self.befehl(c),
            Msg::TasteP(k, mods, p) => { if self.einst.kuerzel_diagnose && (mods.command() || mods.control() || mods.alt() || matches!(k, keyboard::Key::Named(_))) { let c = befehle::BEFEHLE.iter().find(|i| trifft(i.tasten, &k, mods)).map(|i| i.id).unwrap_or("-"); self.diag_zeile(format!("Kürzel: Taste={k:?} physisch={p:?} Cmd={} Umschalt={} Alt={} Strg={} → Befehl {c}", mods.command(), mods.shift(), mods.alt(), mods.control())) } if let Some(l) = tastatur::lernen(&k, &p) { self.layout_gelernt = Some(l) } self.phys = tastatur::physisch_zeichen(&p); let r = self.update_innen(Msg::Taste(k, mods)); self.phys = None; return r }
            Msg::Taste(k, mods) => { self.hinweis.clear();
                if self.splash { self.splash = false; return Task::none() }
                // Dateibaum mit der Tastatur (Fokus in der Seitenleiste, kein Dialog): Pfeile wählen, → / ← klappen auf/zu bzw. gehen hinein/hinaus, Enter öffnet, F2 benennt um, Entf löscht
                if self.fokus_bereich == Bereich::Seitenleiste && self.seite == Seite::Dateien && self.baum.is_none() && !self.palette.offen && self.kontext.is_none() && !mods.command() && !mods.alt() && !mods.control() {
                    if self.baum_taste(&k) { return Task::none() }
                    if let Some(c) = match &k { keyboard::Key::Named(keyboard::key::Named::F2) => Some(Cmd::DateiUmbenennen), keyboard::Key::Named(keyboard::key::Named::Delete) => Some(Cmd::DateiLoeschen), _ => None } { if let Some(sel) = self.baum_sel.clone() { self.ziel = Some(sel); let t = self.befehl(c); self.ziel = None; return t } }
                }
                // Menü-Taste (oder Umschalt+F10): Aktionen des Studios im aktiven Reiter als Kontextmenü – auch dort, wo die rechte Maustaste ein Werkzeug ist
                if self.kontext.is_none() && !self.palette.offen && (matches!(&k, keyboard::Key::Named(keyboard::key::Named::ContextMenu)) || (mods.shift() && matches!(&k, keyboard::Key::Named(keyboard::key::Named::F10)))) {
                    if let Some(rel) = self.name().filter(|_| !self.studio_aktionen().is_empty()) { let pos = iced::Point::new(self.fenster.width * 0.45, self.fenster.height * 0.3); return self.update_innen(Msg::KontextAuf(Art::FlaecheStudio, pos, Some(rel))) }
                }
                if matches!(k, keyboard::Key::Named(keyboard::key::Named::Escape)) && self.wiz.is_some() && !self.palette.offen && self.rueckfrage.is_none() {
                    // Esc im Einrichtungsassistenten: wie „Abbrechen“ (auf der letzten Seite wie „Fertig“)
                    let fertig = self.wiz.as_ref().is_some_and(|w| w.thema() == vc_ide::setup::Topic::Done);
                    return self.wiz_akt(if fertig { assistent::WizAkt::Schliessen } else { assistent::WizAkt::Abbruch });
                }
                if matches!(k, keyboard::Key::Named(keyboard::key::Named::Escape)) { self.palette.offen = false; self.find.offen = false; self.goto.offen = false; self.baum = None; self.kontext = None; self.ziel_menue = false; self.rueckfrage = None; self.export = None; self.import = None; if let Some(e) = self.ed() { e.clear_search() } if let Some(n) = self.name().filter(|n| self.studios.contains_key(n)) { self.studio_aktion(&n, studio::Aktion::AuswahlAus) } self.editor_fokus(); return Task::none() }
                // Menüleiste im Fenster: Alt+Buchstabe öffnet das Menü (D Datei, B Bearbeiten, A Ansicht, P Projekt, H Hilfe; F10 bleibt „Schritt über“), links/rechts wechselt das Menü
                if self.alt_mnemonics() && mods.alt() && !mods.command() && !mods.control() && !mods.shift() {
                    // AltGr (Strg+Alt) bleibt ausgeschlossen (oben: !control); unter macOS liefert Alt+Buchstabe Sonderzeichen, dort zählt die physische Taste
                    let buchstabe = self.phys.or_else(|| if let keyboard::Key::Character(c) = &k { c.to_lowercase().chars().next() } else { None });
                    if let Some(n) = buchstabe.and_then(|b| befehle::MENUES.iter().find(|m| menue_buchstabe(m) == Some(b))) { self.menue_oeffnen(n); return Task::none() }
                }
                if self.kontext.as_ref().is_some_and(|k| k.art == Art::Menue) && !mods.command() && !mods.alt() && !mods.control() {
                    let ziel = self.kontext.as_ref().and_then(|k| k.ziel.clone()).unwrap_or_default(); let (alle, in_unter) = (ziel == "*", ziel.contains('/'));
                    let zeile_msg = self.kontext.as_ref().and_then(|k| { let (z, _) = self.kontext_zeilen(k); z.get(k.sel).filter(|z| z.aktiv).and_then(|z| z.msg.clone()) });
                    match &k {
                        // Pfeil rechts: Untermenü öffnen, sonst nächstes Menü; Pfeil links: Untermenü schließen, sonst voriges Menü
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) if !alle => { if let Some(Msg::MenueAuf(n)) = zeile_msg.filter(|m| matches!(m, Msg::MenueAuf(n) if n.contains('/'))) { self.menue_oeffnen(&n); return self.menue_scrollen() } if !in_unter { self.menue_weiter(1) } return Task::none() }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) if !alle => { if in_unter { if let Some(m) = ziel.split('/').next() { self.menue_oeffnen(m) } } else { self.menue_weiter(-1) } return Task::none() }
                        _ => {} }
                }
                if self.kontext.is_some() {
                    use keyboard::key::Named;
                    match &k {
                        keyboard::Key::Named(Named::ArrowDown) => { self.kontext_pfeil(1); return self.menue_scrollen() }
                        keyboard::Key::Named(Named::ArrowUp) => { self.kontext_pfeil(-1); return self.menue_scrollen() }
                        keyboard::Key::Named(Named::Enter) if self.kontext.as_ref().is_some_and(|k| k.art == Art::FlaecheStudio) => { let m = self.kontext.as_ref().and_then(|k| { let (z, _) = self.kontext_zeilen(k); z.get(k.sel).filter(|z| z.aktiv).and_then(|z| z.msg.clone()) }); if let Some(m) = m { return self.update_innen(m) } return Task::none() }
                        keyboard::Key::Named(Named::Enter) => { if let Some(c) = self.kontext_befehl_unter_auswahl() { return self.update_innen(Msg::KontextWahl(c)) } if let Some(k) = self.kontext.as_ref().filter(|k| k.art == Art::Menue) { let (z, _) = self.kontext_zeilen(k); if let Some(m @ Msg::MenueAuf(_)) = z.get(k.sel).filter(|z| z.aktiv).and_then(|z| z.msg.clone()) { return self.update_innen(m) } } return Task::none() }
                        _ => {}
                    }
                }
                if self.palette.offen { match &k { keyboard::Key::Named(keyboard::key::Named::ArrowDown) => return self.update_innen(Msg::PaletteAuswahl(1)), keyboard::Key::Named(keyboard::key::Named::ArrowUp) => return self.update_innen(Msg::PaletteAuswahl(-1)), _ => {} } }
                if let Some(c) = befehle::BEFEHLE.iter().find(|i| trifft(i.tasten, &k, mods)).map(|i| i.cmd) { if self.doppelt(c, false) { return Task::none() } return self.befehl(c) }
                if !self.palette.offen && !self.find.offen && !self.goto.offen && self.baum.is_none() && self.studio_taste_erlaubt(&k, mods) { if let Some(n) = self.name().filter(|n| self.studios.contains_key(n)) { self.studio_taste(&n, &k, mods); } else if let Some(n) = self.name().filter(|n| self.screens.contains_key(n)) { self.screen_taste(&n, &k, mods); } else if let Some(n) = self.name().filter(|n| self.tilesets.contains_key(n)) { self.tile_taste(&n, &k, mods); } else if let Some(n) = self.name().filter(|n| self.banken.contains_key(n)) { self.bank_taste(&n, &k, mods); } else if let Some(n) = self.name().filter(|n| self.songs.contains_key(n)) { self.song_taste(&n, &k, mods); } }
            }
            Msg::EdIn(r, m) => { self.gruppe_fokus(r); if matches!(m, EdMsg::Click(..) | EdMsg::Kontext(..)) { self.ed_aktiv = true } return self.update_innen(Msg::Ed(m)) }
            Msg::ReiterIn(r, i) => { self.gruppe_fokus(r); return self.update_innen(Msg::Reiter(i)) }
            Msg::SchliesseIn(r, i) => { self.gruppe_fokus(r); return self.update_innen(Msg::Schliesse(i)) }
            Msg::TabDruck(r, i) => { self.marken_sichern(); self.gruppe_fokus(r); if let Some(t) = self.tabs.get(i).cloned() { self.aktiv = i; self.markiere(); self.ziehen = Some(Ziehen { tab: t, start: None, jetzt: iced::Point::ORIGIN }) } }
            Msg::TabUeber(u) => { if self.ziehen.is_none() { self.ueber_tab = u } }
            Msg::Maus(p) => { if let Some(z) = &mut self.ziehen { z.start.get_or_insert(p); z.jetzt = p } }
            Msg::Fenster(s) => self.fenster = s,
            // Kleiner Bildschirm als das Startfenster (1600×1000): maximiert öffnen
            Msg::Bildschirm(id, m) => { if m.is_some_and(|m| m.width < 1600.0 || m.height < 1000.0) { return iced::window::maximize(id, true) } }
            Msg::MausLos => if let Some(z) = self.ziehen.take() {
                let bewegt = z.start.is_some_and(|s| ((z.jetzt.x - s.x).powi(2) + (z.jetzt.y - s.y).powi(2)).sqrt() > 6.0);
                if bewegt { let (x, y) = (z.jetzt.x, z.jetzt.y);
                    let (x0, x1) = (if self.layout.offen(Bereich::Seitenleiste) { 260.0 } else { 38.0 }, self.fenster.width / self.einst.zoom - if self.layout.offen(Bereich::Panel) { self.panel_breite() + 38.0 } else { 38.0 });
                    if y > 48.0 && x >= x0 && x <= x1 {
                        let ziel = self.tab_unter(x, y, x0, x1);
                        match ziel { Some((r, i)) => self.tab_verschieben(z.tab, r, Some(i)), None => { let rechts = if self.andere.is_some() { x > (x0 + x1) / 2.0 } else { x > x0 + 0.65 * (x1 - x0) }; self.tab_verschieben(z.tab, rechts, None) } }
                    }
                }
            },
            Msg::StreifenKlick(p) => {
                if self.panel_offen() && self.panel_seite == p { self.layout.minimieren(Bereich::Panel); self.layout_merken() } else { return self.panel_oeffnen(p) }
            }
            Msg::BereichAkt(b, a) => { match a { BA::Min => { self.layout.minimieren(b); } BA::Max => self.layout.maximieren(b), BA::Wieder => self.layout.wiederherstellen() } self.layout_merken() }
            Msg::SpalteAuf(s) => { self.seite = s; self.layout.aufklappen(Bereich::Seitenleiste); self.layout_merken(); if s == Seite::Suche { return self.text_fokus("suche") } }
            Msg::BereichAuf(b) => { self.layout.aufklappen(b); self.layout_merken() }
            Msg::PanelWahl(p) => { self.panel_seite = p; if p == PanelSeite::Karte { self.karte_laden() } if p == PanelSeite::Hilfe && self.hilfe.themen.is_empty() { self.hilfe.themen = vc_ide_model::helpstate::topics() } }
            Msg::HilfeSuche(q) => { self.hilfe.treffer = vc_ide_model::helpstate::search(&self.data, &self.hilfe_ziel(), &q); self.hilfe.suche = q; self.hilfe.doc = None }
            Msg::HilfeOeffne(k) => { let (ziel, sprache) = (self.hilfe_ziel(), self.hilfe_sprache()); if let Some(d) = vc_ide_model::helpstate::open(&self.data, &ziel, sprache, &k) { if let Some(alt) = self.hilfe.doc.take() { self.hilfe.hist.push(alt) } self.hilfe.doc = Some(d) } }
            Msg::HilfeImReiter => { let k = self.hilfe.doc.as_ref().map(|d| d.id.clone()).unwrap_or_default(); self.tab_oeffnen(Tab::Hilfe(k)) }
            Msg::HilfeZurueck => { self.hilfe.doc = self.hilfe.hist.pop() }
            Msg::ZeichenSuche(q) => self.zeichen_suche = q,
            Msg::ZeichenEinfuegen(w) => self.zeichen_einfuegen(w),
            Msg::BaumNeu(ordner, art) => { self.baum = Some(BaumDialog { art: BaumArt::Neu { ordner, art: NeuArt::Asm, profile: vec![], profil: 0 }, text: String::new(), fehler: String::new() }); self.neu_art_waehlen(art); return self.text_fokus("baum") }
            Msg::BaumOrdner(ordner) => { self.baum = Some(BaumDialog { art: BaumArt::Ordner(ordner), text: String::new(), fehler: String::new() }); return self.text_fokus("baum") }
            Msg::BaumZugStart(rel, ordner) => { self.baum_sel = Some(rel.clone()); self.baum_zug = Some(BaumZug { quelle: rel, ordner, bewegt: false, ziel: None }) }
            Msg::BaumZugUeber(rel) => { let ist_ordner = project::pfad(&self.root, &rel).is_dir(); if let Some(g) = &mut self.baum_zug { if rel != g.quelle { g.bewegt = true; g.ziel = Some(if ist_ordner { rel } else { rel.rsplit_once('/').map_or(String::new(), |(o, _)| o.to_string()) }) } else if g.bewegt { g.ziel = None } } }
            Msg::BaumZugLos(rel) => { if let Some(g) = self.baum_zug.take() {
                if !g.bewegt || g.quelle == rel { return self.update_innen(if g.ordner { Msg::BaumKlapp(rel) } else { Msg::OeffneDatei(rel) }) }
                let ziel = g.ziel.unwrap_or_else(|| if project::pfad(&self.root, &rel).is_dir() { rel.clone() } else { rel.rsplit_once('/').map_or(String::new(), |(o, _)| o.to_string()) });
                self.baum_verschieben(&g.quelle, &ziel) } }
            Msg::BaumZugAbbruch => { self.baum_zug = None; for e in self.editoren.values_mut() { e.einfuegemarke = None } }
            Msg::BaumOrdnerWahl(o) => { if let Some(b) = &mut self.baum { match &mut b.art { BaumArt::Neu { ordner, .. } => *ordner = o, BaumArt::Ordner(ordner) => *ordner = o, _ => {} } b.fehler.clear() } return self.text_fokus("baum") }
            Msg::BaumArtWahl(art) => { self.neu_art_waehlen(art); return self.text_fokus("baum") }
            Msg::BaumProfil(i) => { if let Some(BaumDialog { art: BaumArt::Neu { profile, profil, .. }, .. }) = &mut self.baum { if i < profile.len() { *profil = i } } }
            Msg::KontextAuf(art, pos, ziel) => { self.kontext = Some(Kontext { art, pos, ziel, sel: 0 }); self.kontext_sel_setzen() }
            Msg::KontextReiter(r, i, pos) => { self.marken_sichern(); self.gruppe_fokus(r); if i < self.tabs.len() { self.aktiv = i; self.markiere() } self.kontext = Some(Kontext { art: Art::Reiter, pos, ziel: None, sel: 0 }); self.kontext_sel_setzen() }
            Msg::KontextZu => self.kontext = None,
            Msg::MenueAuf(n) => { let offen = self.kontext.as_ref().is_some_and(|k| k.art == Art::Menue && k.ziel.as_deref() == Some(n.as_str())); if offen { self.kontext = None } else { self.menue_oeffnen(&n) } }
            Msg::SetMenueleiste(b) => { self.einst.menueleiste = b; self.einst.speichern() }
            Msg::KontextMsg(m) => { self.kontext = None; return self.update_innen(*m) }
            Msg::KontextWahl(c) => { let ziel = self.kontext.take().and_then(|k| if k.art == Art::Menue { None } else { k.ziel }); self.ziel = ziel; let t = self.befehl(c); self.ziel = None; return t }
            Msg::Einfuegen(t) => { if let (Some(t), Some(ed)) = (t, self.ed()) { ed.insert(&t.replace("\r\n", "\n")) } self.editor_fokus() }
            Msg::BaumUmbenennen => match self.ziel.take().or_else(|| self.name()) { Some(n) => { let kurz = n.rsplit('/').next().unwrap_or(&n).to_string(); self.baum = Some(BaumDialog { art: BaumArt::Umbenennen(n.clone()), text: kurz, fehler: String::new() }); return self.text_fokus("baum") } None => self.log.push(t::BAUM_KEINE_DATEI.into()) },
            Msg::BaumLoeschen => match self.ziel.take().or_else(|| self.name()) { Some(n) => self.baum = Some(BaumDialog { art: BaumArt::Loeschen(n), text: String::new(), fehler: String::new() }), None => self.log.push(t::BAUM_KEINE_DATEI.into()) },
            Msg::BaumText(x) => { if let Some(b) = &mut self.baum { b.text = x; b.fehler.clear() } }
            Msg::BaumAbbruch => self.baum = None,
            Msg::BaumOk => self.baum_ausfuehren(),
            Msg::OeffneDatei(n) => { self.baum_ziel = n.rsplit_once('/').map_or(String::new(), |(o, _)| o.to_string()); self.oeffne(&n) }
            Msg::Reiter(i) => { self.marken_sichern(); self.aktiv = i; self.markiere() }
            Msg::Schliesse(i) if matches!(self.tabs.get(i), Some(Tab::Datei(n)) if self.ist_scratch(n)) => { let Some(Tab::Datei(n)) = self.tabs.get(i).cloned() else { return Task::none() }; if self.editoren.get(&n).is_some_and(|e| !e.text().trim().is_empty()) { self.rueckfrage = Some(Rueckfrage::ScratchWeg(n)) } else { self.scratch_verwerfen(&n) } }
            Msg::Schliesse(i) => { self.marken_sichern(); if i < self.tabs.len() { self.tabs.remove(i); self.aktiv = self.aktiv.min(self.tabs.len().saturating_sub(1)); if self.tabs.is_empty() { if let Some((t, a)) = self.andere.take() { self.tabs = t; self.aktiv = a; self.fokus_rechts = false } } self.markiere() } }
            Msg::Schublade(i) => { self.layout.aufklappen(Bereich::Schublade); self.layout_merken(); self.schublade = i }
            Msg::BauFertig(r) => { self.baut = false; let ok = self.bau_ende(r); self.markiere(); let d = std::mem::replace(&mut self.danach, Danach::Nichts); if ok { match d { Danach::Start => return self.starte(), Danach::Debug => self.debug_start(), Danach::Nichts => {} } } }
            Msg::DebugTick => self.debug_abgleich(),
            Msg::MemText(t) => self.mem_text = t,
            Msg::MemGo => { if let Some(d) = &self.dbg { let sym = self.info.as_ref().map(|i| i.symbols.clone()).unwrap_or_default(); match debug::speicher_adresse(&self.mem_text, &sym) { Ok(a) => d.send(debug::Cmd::Mem(a)), Err(e) => self.log.push(e) } } }
            Msg::BeobText(t) => self.beob_text = t,
            Msg::BeobAdd => { let e = self.beob_text.trim().to_string(); if !e.is_empty() && !self.beob.contains(&e) { self.beob.push(e); self.beob_text.clear(); if let Some(d) = &self.dbg { d.send(debug::Cmd::Watches(self.beob.clone())) } } }
            Msg::BeobDel(i) => { if i < self.beob.len() { self.beob.remove(i); if let Some(d) = &self.dbg { d.send(debug::Cmd::Watches(self.beob.clone())) } } }
            Msg::PunktDel(id) => { if let Some(d) = &self.dbg { d.send(debug::Cmd::DelPoint(id)) } }
            Msg::KarteSeite(p) => self.karte.sel = p,
            Msg::KarteFilter(f) => self.karte.filter = f,
            Msg::KartePreset(id) => self.karte.preset = Some(id),
            Msg::DebugSpalte(i) => self.debug_spalte = i.min(3),
            Msg::TrennerStart => self.trenner = true,
            Msg::TrennerMaus(p) => { if self.trenner { let h = self.fenster.height / self.einst.zoom; let nutz = (h - 52.0 - 26.0).max(100.0); self.einst.debug_hoehe = ((h - 26.0 - p.y) / nutz).clamp(0.2, 0.7) } }
            Msg::TrennerLos => { if self.trenner { self.trenner = false; self.einst.speichern() } }
            Msg::HexAkt(rel, a) => { self.fokus_auf(&rel); self.hex_aktion(&rel, a) }
            Msg::DiskAkt(rel, a) => { self.fokus_auf(&rel); return self.disk_aktion(&rel, a) }
            Msg::DiskPfad(rel, art, p) => self.disk_pfad(&rel, art, p),
            Msg::DiskDateien(rel, v) => self.disk_dateien(&rel, v),
            Msg::DiskFertig(rel, r) => self.disk_fertig(&rel, r),
            Msg::SongTick => return self.song_tick(),
            Msg::SongKonv(rel, rmt) => { self.fokus_auf(&rel); return Task::perform(async move { rfd::AsyncFileDialog::new().set_title(if rmt { "RMT-Modul wählen" } else { "Song aus anderem System wählen" }).add_filter(if rmt { "RMT-Modul" } else { "SID-Datei (.sid), GoatTracker (.sng) oder Song (.vcsong)" }, if rmt { &["rmt"][..] } else { &["sng", "sid", "vcsong"][..] }).pick_file().await.map(|f| f.path().to_path_buf()) }, move |p| Msg::SongDatei(rel.clone(), rmt, p)) }
            Msg::SongDatei(rel, rmt, p) => { if let Some(p) = p { self.song_import_oeffnen(&rel, rmt, p) } }
            Msg::SongImpName(n) => { if let Some(d) = &mut self.song_import { d.name = n; d.fehler.clear() } }
            Msg::SongImpSub(x) => { if let Some(d) = &mut self.song_import { if let Some((_, songs, _)) = &d.quelle.sid { d.subtune = (d.subtune as i32 + x).clamp(1, *songs as i32) as u16 } } }
            Msg::SongImpSek(x) => { if let Some(d) = &mut self.song_import { d.sekunden = (d.sekunden as i32 + x).clamp(5, 600) as u32 } }
            Msg::SongImpAbbruch => self.song_import = None,
            Msg::SongImpOk => { if let Some(job) = self.song_import_job() { if let Some(d) = &mut self.song_import { d.laeuft = true } return Task::perform(async move { tokio::task::spawn_blocking(job).await.unwrap_or_else(|e| Err(e.to_string())) }, |r| Msg::SongImpFertig(songimport_ergebnis::Erg(r))) } }
            Msg::SongImpFertig(r) => self.song_import_fertig(r.0),
            Msg::SongAkt(rel, songstudio::GAkt::Konvertieren) => return self.update_innen(Msg::SongKonv(rel, false)),
            Msg::SongAkt(rel, songstudio::GAkt::RmtImport) => return self.update_innen(Msg::SongKonv(rel, true)),
            Msg::SongAkt(rel, a) => { self.fokus_auf(&rel); self.song_aktion(&rel, a) }
            Msg::SongMaus(rel, m) => { self.fokus_auf(&rel); self.song_maus(&rel, m) }
            Msg::BankAkt(rel, a) => { self.fokus_auf(&rel); self.bank_aktion(&rel, a) }
            Msg::BankMaus(rel, m) => { self.fokus_auf(&rel); self.bank_maus(&rel, m) }
            Msg::TileAkt(rel, a) => { self.fokus_auf(&rel); self.tile_aktion(&rel, a) }
            Msg::TileMaus(rel, m) => { self.fokus_auf(&rel); self.tile_maus(&rel, m) }
            Msg::ScreenAkt(rel, a) => { self.fokus_auf(&rel); self.screen_aktion(&rel, a) }
            Msg::ScreenMaus(rel, m) => { self.fokus_auf(&rel); self.screen_maus(&rel, m) }
            Msg::GruppeKlick(r) => { self.gruppe_fokus(r); self.ed_aktiv = false; self.fokus_bereich = Bereich::Dokument }
            Msg::StudioMaus(rel, m) => { self.fokus_auf(&rel); self.studio_maus(&rel, m) }
            Msg::StudioAkt(rel, studio::Aktion::PngLaden) => { self.fokus_auf(&rel); return Task::perform(async { rfd::AsyncFileDialog::new().set_title("PNG-Bild laden").add_filter("Bild", &["png"]).pick_file().await.map(|f| f.path().to_path_buf()) }, move |p| Msg::PngGewaehlt(rel.clone(), p)) }
            Msg::PngGewaehlt(rel, p) => { if let Some(p) = p { self.png_laden(&rel, &p) } }
            Msg::StudioAkt(rel, a) => { self.fokus_auf(&rel); self.studio_aktion(&rel, a) }
            Msg::StudioTick => {}
            Msg::PaletteAkt(rel, a) => { self.fokus_auf(&rel); self.palette_aktion(&rel, a) }
            Msg::ImportAn(i, b) => { let root = self.root.clone(); if let Some(z) = self.import.as_mut().and_then(|d| d.zeilen.get_mut(i)) { z.an = b && importdlg::uebernehmbar(&root, z) } }
            Msg::ImportKopie(i) => { let root = self.root.clone(); if let Some(z) = self.import.as_mut().and_then(|d| d.zeilen.get_mut(i)) { z.plan.name = vc_ide::importfiles::unique_name(&root, &z.plan.dir, &z.plan.name); z.an = importdlg::uebernehmbar(&root, z) } }
            Msg::ImportName(i, n) => { let root = self.root.clone(); if let Some(z) = self.import.as_mut().and_then(|d| d.zeilen.get_mut(i)) { z.plan.name = n; z.an = importdlg::uebernehmbar(&root, z) } }
            Msg::ImportOrdner(i, o) => { if let Some(z) = self.import.as_mut().and_then(|d| d.zeilen.get_mut(i)) { z.plan.dir = o.trim().trim_matches('/').to_string() } }
            Msg::ImportKonflikt(i, c) => { if let Some(z) = self.import.as_mut().and_then(|d| d.zeilen.get_mut(i)) { z.plan.conflict = c } }
            Msg::ImportOk => self.import_ausfuehren(),
            Msg::ImportAbbruch => self.import = None,
            Msg::ExportFormat(f) => { if let Some(d) = &mut self.export { d.format = f; d.tab = "text".into() } self.export_rechnen() }
            Msg::ExportName(n) => { if let Some(d) = &mut self.export { d.name = n } self.export_rechnen() }
            Msg::ExportAdresse(a) => { if let Some(d) = &mut self.export { d.adresse = a } self.export_rechnen() }
            Msg::ExportTab(t_) => { let daten = t_ == "data"; if let Some(d) = &mut self.export { d.tab = t_ } if daten { self.export_data_tab() } }
            Msg::ExportAus(a) => return self.export_ausfuehren(a),
            Msg::Zeichen(a) => return self.zeichen_akt(a),
            Msg::MdLink(u) => { if let Some(k) = u.strip_prefix("help:").or_else(|| u.strip_prefix("hilfe:")) { let k = k.split('#').next().unwrap_or(k).to_string(); if matches!(self.tabs.get(self.aktiv), Some(Tab::Hilfe(_))) { self.tabs[self.aktiv] = Tab::Hilfe(k); return Task::none() } return self.update_innen(Msg::HilfeOeffne(k)) } }
            Msg::HexVorschlag(a) => return self.hexv_akt(a),
            Msg::MdBildGewaehlt(p) => self.md_bild_einfuegen(p),
            Msg::ExportKachel(a) => { if let Some(d) = &mut self.export { if let Some(l) = &mut d.kachel { match a { exportdlg::KachelAkt::Tiles(b) => l.interleaved = b, exportdlg::KachelAkt::Karte(b) => l.map_columns = b, exportdlg::KachelAkt::Index(b) => l.index16 = b } } } self.export_rechnen() }
            Msg::DisHaltepunkt(a) => {
                // Klick auf den Rand des Disassemblers: Haltepunkt in der Quellzeile der Adresse umschalten (der Editor ist maßgeblich)
                let ziel = self.info.as_ref().and_then(|i| i.zeile_bei(a)).map(|(f, n)| (f.to_string(), n));
                match ziel { Some((f, n)) => { if let Some(e) = self.editoren.get_mut(&f) { let l = n - 1; if !e.breakpoints.remove(&l) { e.breakpoints.insert(l); } } } None => self.log.push(format!("Zu ${a:04X} gibt es keine Quellzeile: dort lässt sich kein Haltepunkt setzen.")) }
            }
            Msg::RueckfrageNein => self.rueckfrage = None,
            Msg::RueckfrageJa => match self.rueckfrage.take() {
                Some(Rueckfrage::TestDebug(f, n)) => { self.debug_ende(); self.test_debug_los(f, n) }
                Some(Rueckfrage::DebugNeu) => { self.debug_ende(); return self.bauen(Danach::Debug) }
                Some(Rueckfrage::ScratchWeg(n)) => self.scratch_verwerfen(&n),
                Some(Rueckfrage::Kachelkarte(n)) => self.kachelkarte_umwandeln(&n),
                Some(Rueckfrage::KonfigImport(p, _)) => self.konfig_import_ja(&p),
                Some(Rueckfrage::KonfigReset) => self.konfig_reset_ja(),
                None => {}
            },
            Msg::EmuBild => { if let Some(d) = &self.dbg { d.send(debug::Cmd::Frame) } }
            Msg::EmuAuto(b) => { self.emu_auto = b; self.emu_modus_senden() }
            Msg::EmuLive(b) => { self.emu_live = b; self.emu_modus_senden() }
            Msg::EmuQuelle(i) => { if let Some(d) = &self.dbg { d.send(debug::Cmd::FrameSource(i)) } }
            Msg::EmuText(t) => self.emu_text = t,
            Msg::EmuSenden => { if !self.emu_text.is_empty() { if let Some(d) = &self.dbg { d.send(debug::Cmd::Text(std::mem::take(&mut self.emu_text))) } } }
            Msg::TestAlle => return self.tests_starten(vc_ide::tests::suite::Only::All),
            Msg::TestFehler => { let f: Vec<(PathBuf, String)> = self.tests.suite.groups.iter().flat_map(|g| g.items.iter().filter(|i| matches!(i.outcome.state, vc_ide::tests::State::Failed | vc_ide::tests::State::TimedOut)).map(move |i| (g.file.clone(), i.outcome.name.clone()))).filter_map(|(f, n)| f.map(|f| (f, n))).collect(); return self.tests_starten(vc_ide::tests::suite::Only::Failed(f)) }
            Msg::TestStopp => self.tests.abbrechen(),
            Msg::TestBackend => self.tests.backend_weiter(),
            Msg::TestFilter(t) => self.tests.filter = t,
            Msg::TestNurFehler(b) => self.tests.nur_fehler = b,
            Msg::TestWahl(f, n) => self.tests.sel = Some((f, n)),
            Msg::TestQuelle(d, z) => self.zeige(&d, z),
            Msg::TestTick => self.tests.abholen(),
            Msg::TestDebug(f, n) => self.test_debug_start(f, n),
            Msg::SnapErkennen => { if let Some(d) = &self.dbg { d.send(debug::Cmd::SnapDetect(self.system_id())) } }
            Msg::SnapName(n) => self.snap_name = n,
            Msg::SnapSprites(b) => self.snap_sprites = b,
            Msg::SnapKacheln(b) => self.snap_tiles = b,
            Msg::SnapRom(b) => self.snap_rom = b,
            Msg::SnapSpeichern => {
                if let (Some(d), Some(Ok(det))) = (&self.dbg, &self.snap.detected) {
                    let name: String = self.snap_name.trim().chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
                    let name = if name.is_empty() { "momentaufnahme".to_string() } else { name };
                    let opts = vc_ide::snapshot::Opts { screen: det.supported.then_some(det.screen), charset: det.charset, rom: self.snap_rom && det.rom_off.is_some(), sprites: self.snap_sprites && !det.sprites.is_empty(), tiles: self.snap_tiles, name };
                    d.send(debug::Cmd::SnapSave { det: det.clone(), opts, root: self.root.clone(), data: self.data.clone() });
                }
            }
            Msg::ZielMenue => self.ziel_menue = !self.ziel_menue,
            Msg::Set(a) => return self.set_akt(a),
            Msg::StartSeite(a) => return self.start_akt(a),
            Msg::FensterZu => return self.befehl(Cmd::Beenden),
            Msg::BeendenAntwort(w) => return self.beenden_antwort(w),
            Msg::NeuProj(a) => return self.np_akt(a),
            Msg::Kopieren(a) => return self.kop_akt(a),
            Msg::ProjEinst(a) => return self.pe_akt(a),
            Msg::Wiz(a) => return self.wiz_akt(a),
            Msg::Info(a) => return self.info_akt(a),
            Msg::SongScroll(r, x, y, w, h) => self.song_scroll(&r, x, y, w, h),
            Msg::ScratchTick => self.scratch_sichern(),
            Msg::ScratchRest(w) => self.scratch_wiederherstellen(w),
            Msg::KopfMenue => self.kopf_menue = !self.kopf_menue,
            Msg::KopfWahl(c) => { self.kopf_menue = false; return self.befehl(c) }
            Msg::ZielWahl(id) => { self.ziel_menue = false; self.log.push(format!("Ziel: {}", self.ziel_name(&id))); self.run_on = id }
            // Endet der Emulator in den ersten Sekunden mit Fehler, steht seine letzte Ausgabe mit einem Hinweis im Protokoll (z. B. VICE ohne ROM-Dateien)
            Msg::StartFehler(zeilen) => { if !zeilen.is_empty() { for m in zeilen { for l in m.lines() { self.log.push(l.to_string()) } } self.schublade = 0; self.einst.schublade = true } }
            Msg::Gestartet(Ok(c)) if !cfg!(test) => { self.log.push(format!("Gestartet: {c}")); return Task::perform(async { tokio::task::spawn_blocking(|| vc_ide::launch::wait_early_exit(std::time::Duration::from_secs(8))).await.unwrap_or_default() }, Msg::StartFehler) }
            Msg::Gestartet(r) => match r { Ok(c) => self.log.push(format!("Gestartet: {c}")), Err(e) => { self.log.push(format!("Start fehlgeschlagen: {}", debug::freundlich(&e))); if debug::freundlich(&e) != e { self.log.push(format!("  Einzelheit: {e}")) } self.schublade = 0; self.einst.schublade = true } },
            Msg::GotoProblem(i) => { if let Some(p) = self.probleme.get(i).cloned() { self.zeige(&p.datei, p.zeile) } }
            Msg::PaletteFrage(q) => { self.palette.frage = q; self.palette.auswahl = 0 }
            Msg::PaletteAuswahl(d) => { let n = self.palette_eintraege().len().max(1) as i32; self.palette.auswahl = (self.palette.auswahl as i32 + d).rem_euclid(n) as usize }
            Msg::PaletteLos(i) => { let e = self.palette_eintraege().get(i).cloned(); self.palette.offen = false; match e { Some(Eintrag::Befehl(c)) => return self.befehl(c), Some(Eintrag::Studio(_, _, m)) => return self.update_innen(*m), Some(Eintrag::Datei(n)) => self.oeffne(&n), Some(Eintrag::Projekt(p, _)) => { self.projekt_oeffnen(p); } None => {} } }
            Msg::SeiteWahl(s) => self.seite = s,
            Msg::BaumKlapp(d) => { self.baum_ziel = d.clone(); if !self.baum_zu.remove(&d) { self.baum_zu.insert(d); } self.baum_zustand_sichern() }
            Msg::BaumFilter(f) => self.baum_filter = f,
            Msg::BaumNeuLaden => self.projekt_neu_laden(),
            Msg::BaumAlleZu => { self.baum_zu = self.proj.dirs.iter().cloned().collect(); self.baum_zustand_sichern() }
            Msg::SucheFrage(q) => { self.suche_frage = q; self.suche_fertig = false }
            Msg::SucheLos => return self.suche_starten(),
            Msg::SucheFertig(v) => { self.sucht = false; self.suche_fertig = true; self.funde = v }
            Msg::GotoFund(i) => { if let Some(f) = self.funde.get(i).cloned() { self.zeige(&f.rel, f.zeile) } }
            Msg::FindFrage(q) => { self.find.frage = q; self.find_aktualisieren(true) }
            Msg::ErsetzFrage(q) => self.find.mit = q,
            Msg::FindSchritt(d) => { if let Some(e) = self.ed() { e.step_match(d); } self.find_info() }
            Msg::ErsetzeEins => { let (q, w, c) = (self.find.frage.clone(), self.find.mit.clone(), self.find.gross_klein); if let Some(e) = self.ed() { e.replace_current(&w, &q, c) } self.find_info() }
            Msg::ErsetzeAlle => { let (q, w, c) = (self.find.frage.clone(), self.find.mit.clone(), self.find.gross_klein); let n = self.ed().map(|e| e.replace_all(&w, &q, c)).unwrap_or(0); self.find.info = format!("{n} ersetzt") }
            Msg::FindGrossKlein => { self.find.gross_klein = !self.find.gross_klein; self.find_aktualisieren(false) }
            Msg::FindZu => { self.find.offen = false; if let Some(e) = self.ed() { e.clear_search() } }
            Msg::GotoZeile(l) => { if let Some(e) = self.ed() { let l = l.min(e.lines.len() - 1); e.set_cursor(Pos { line: l, col: 0 }, false); e.center_on(l) } }
            Msg::GotoText(s) => self.goto.text = s,
            Msg::GotoLos => { if let Ok(n) = self.goto.text.trim().parse::<usize>() { if let Some(e) = self.ed() { let l = n.saturating_sub(1).min(e.lines.len() - 1); e.set_cursor(Pos { line: l, col: 0 }, false); e.center_on(l) } self.goto.offen = false } }
            Msg::SetSchema(s) => { self.einst.schema = s; self.einst.speichern() }
            Msg::SetZoom(z) => { self.einst.zoom = z; self.einst.begrenze(); self.einst.speichern() }
            Msg::SetSchrift(z) => { self.einst.editor_schrift = z; self.einst.begrenze(); self.einst.speichern() }
            Msg::Zahl(a) => return self.zahlen_aktion(a),
            Msg::SplashZu => { self.splash = false }
            Msg::SetSplash(b) => { self.einst.splash = b; self.einst.speichern() }
            Msg::KonfigWahl(a) => return self.konfig_dialog(a),
            Msg::KonfigPfad(a, p) => self.konfig_pfad(a, p),
            Msg::KonfigResetFrage => { self.rueckfrage = Some(Rueckfrage::KonfigReset) }
            Msg::MdMehr => { self.md_grenze += mdansicht::BLOECKE }
            Msg::MdUmschalten(n) => { if !self.md_quelltext.remove(&n) { self.md_quelltext.insert(n); } }
            Msg::PfadWaehlen(art, wert, s) => return self.pfad_waehlen(art, wert, s),
            Msg::PfadGewaehlt(art, s, p) => return self.pfad_gewaehlt(art, s, p),
            Msg::SetTakt(b) => { self.einst.takt_anzeige = b; self.einst.speichern() }
            Msg::SetLayout(l) => { self.einst.tastaturlayout = l; self.einst.speichern() }
            Msg::SetKlavierTasten(b) => { self.einst.tasten_auf_klaviatur = b; self.einst.speichern() }
            Msg::SetDiagnose(b) => { self.einst.kuerzel_diagnose = b; self.einst.speichern() }
            Msg::SetPretty(b) => { self.einst.pretty_zeile = b; self.einst.speichern() }
            Msg::SetStudioTasten(b) => { self.einst.studio_einzeltasten = b; self.einst.speichern() }
            Msg::SetUmbruch(u) => { self.einst.umbruch = u; for e in self.editoren.values_mut() { e.wrap = u; e.version += 1 } self.einst.speichern() }
            Msg::Abgelegt(p) => { if self.rom_seite_offen() { self.rom_abgelegt(p) } else { self.abgelegt(p) } }
            Msg::Lsp(a) => self.lsp_antwort(a),
            Msg::Geoeffnet(p) => { if let Some(p) = p { if self.wechsel_noetig(&p) { let l = self.ungespeichert(); if !l.is_empty() { self.beenden_frage = Some((l, Folge::Projekt(p))); return Task::none() } } self.projekt_oeffnen(p); } }
        }
        Task::none()
    }

    fn oeffne_pfad(&mut self, p: PathBuf) {
        if let Ok(rel) = p.strip_prefix(&self.root) { let r = rel.to_string_lossy().into_owned(); self.oeffne(&r) }
        else if let Ok(txt) = std::fs::read_to_string(&p) { let n = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or("Datei".into()); let mut e = Editor::new(&txt); e.dirty = false; self.editoren.insert(n.clone(), e); self.tab_oeffnen(Tab::Datei(n)); }
        else { self.log.push(format!("{}: nicht lesbar", p.display())) }
    }
    fn zeige(&mut self, datei: &str, zeile: usize) {
        self.oeffne(datei); self.markiere();
        if let Some(e) = self.ed() { let l = zeile.saturating_sub(1).min(e.lines.len() - 1); e.set_cursor(Pos { line: l, col: 0 }, false); e.center_on(l) }
    }
    fn find_aktualisieren(&mut self, springen: bool) {
        let (q, c) = (self.find.frage.clone(), self.find.gross_klein);
        if let Some(e) = self.ed() { e.set_search(&q, c); if springen && !e.matches.is_empty() { let i = e.current_match.unwrap_or(0) as i32; e.current_match = None; let _ = i; e.step_match(1); } }
        self.find_info()
    }
    fn find_info(&mut self) { let q = self.find.frage.clone(); let (n, i) = self.ed().map(|e| (e.matches.len(), e.current_match.map(|i| i + 1))).unwrap_or((0, None)); self.find.info = if q.is_empty() { String::new() } else if n == 0 { t::KEIN_TREFFER.into() } else { format!("{} von {n}", i.unwrap_or(0)) } }
    fn ed_nachricht(&mut self, e: EdMsg) {
        let aendert = matches!(e, EdMsg::Insert(_) | EdMsg::Backspace | EdMsg::Delete | EdMsg::Enter | EdMsg::Tab | EdMsg::Undo | EdMsg::Redo);
        let tippt = matches!(&e, EdMsg::Insert(s) if s.chars().count() == 1); let mut hover_anfrage = None; let mut kontext_offen: Option<iced::Point> = None;
        if let EdMsg::Ablegen(k) = e { self.quelltext_ablegen(k); return }
        let Some(ed) = self.ed() else { return };
        let vor = (ed.cursor.line, ed.lines.len()); let text_aenderung = matches!(&e, EdMsg::Insert(_) | EdMsg::Backspace | EdMsg::Delete | EdMsg::Enter | EdMsg::Tab); let von_hand = text_aenderung || matches!(&e, EdMsg::Move { .. } | EdMsg::Home(_) | EdMsg::End(_) | EdMsg::Page(..) | EdMsg::Click(..)); let rueckgaengig = matches!(&e, EdMsg::Undo | EdMsg::Redo);
        let vokabular: Vec<&str> = vc_gui_editor::highlight::MNEMONICS.iter().chain(vc_gui_editor::highlight::DIRECTIVES.iter()).copied().collect();
        match e {
            EdMsg::Insert(s) => { ed.insert(&s); if s.chars().count() == 1 { ed.update_completion(&vokabular) } else { ed.completion = None } }
            EdMsg::Backspace => { ed.backspace(); ed.completion = None } EdMsg::Delete => ed.delete(), EdMsg::Enter => { ed.completion = None; ed.enter() } EdMsg::Tab => ed.insert("\t"),
            EdMsg::Move { dx, dy, extend } => { if dy != 0 { ed.move_v(dy, extend) } else { ed.move_h(dx, extend) } }
            EdMsg::Home(x) => ed.home(x), EdMsg::End(x) => ed.end(x),
            EdMsg::Page(d, x) => { let n = (ed.view_rows as i32 - 2).max(1) * d; ed.move_v(n, x); ed.scroll_by(n as f32) }
            EdMsg::SelectAll => ed.select_all(), EdMsg::Undo => ed.undo(), EdMsg::Redo => ed.redo(),
            EdMsg::Click(p, shift, dbl) => { ed.center_pending = None; if dbl { let ch: Vec<char> = ed.lines[p.line].chars().collect(); let (mut a, mut b) = (p.col.min(ch.len()), p.col.min(ch.len())); while a > 0 && (ch[a - 1].is_alphanumeric() || ch[a - 1] == '_') { a -= 1 } while b < ch.len() && (ch[b].is_alphanumeric() || ch[b] == '_') { b += 1 } ed.set_cursor(Pos { line: p.line, col: a }, false); ed.set_cursor(Pos { line: p.line, col: b }, true) } else { ed.set_cursor(p, shift) } }
            EdMsg::Drag(p) => { let a = ed.anchor.unwrap_or(ed.cursor); ed.anchor = Some(a); ed.cursor = p; ed.want_col = vc_gui_editor::core::vis_col(&ed.lines[p.line], p.col) }
            EdMsg::Breakpoint(l) => { if !ed.breakpoints.remove(&l) { ed.breakpoints.insert(l); } }
            EdMsg::Fold(l) => ed.toggle_fold(l), EdMsg::Scroll(r) => { ed.center_pending = None; ed.scroll_by(r) }
            EdMsg::Resize(c, r) => { ed.resize(c, r); ed.version += 1 }
            EdMsg::CompletionMove(d) => { if let Some(c) = &mut ed.completion { c.sel = (c.sel as i32 + d).rem_euclid(c.items.len() as i32) as usize } }
            EdMsg::CompletionAccept => { ed.accept_completion(); } EdMsg::CompletionClose => ed.completion = None,
            EdMsg::HScroll(v) => ed.set_hscroll(v), EdMsg::VScroll(r) => { ed.center_pending = None; let max = (ed.total_rows() as f32 - 3.0).max(0.0); ed.scroll_row = r.clamp(0.0, max) } EdMsg::HoverXY(_) => {} EdMsg::Kontext(p, pt) => { if ed.selection().map_or(true, |(a, b)| p < a || p > b) { ed.set_cursor(p, false) } kontext_offen = Some(pt) } EdMsg::Hover(None) => ed.hover = None, EdMsg::Hover(Some(p)) => hover_anfrage = Some(p),
            EdMsg::AblageUeber(m) => ed.einfuegemarke = m, EdMsg::Ablegen(_) => {}
            EdMsg::Preedit(p) => ed.preedit = p, EdMsg::ToggleWrap => { ed.wrap = !ed.wrap; ed.version += 1 }
        }
        self.pretty_pruefen(text_aenderung, rueckgaengig, von_hand, vor);
        if aendert && self.find.offen && !self.find.frage.is_empty() { let (q, c) = (self.find.frage.clone(), self.find.gross_klein); if let Some(ed) = self.ed() { ed.set_search(&q, c) } self.find_info() }
        if let Some(pt) = kontext_offen { self.kontext = Some(Kontext { art: Art::Editor, pos: pt, ziel: None, sel: 0 }); self.kontext_sel_setzen() }
        if tippt { self.vorschlag_noetig = true }
        if aendert { if let Some(ed) = self.ed() { ed.hover = None } }
        if let Some(p) = hover_anfrage {
            // Fehlerzeile unter der Maus: die Meldung steht sofort da (der Sprachdienst ergänzt sie)
            if let Some(n) = self.name() { let d = self.diagnosen.get(&n).and_then(|v| v.iter().find(|d| d.zeile == p.line).cloned()); if let (Some(d), Some(e)) = (d, self.editoren.get_mut(&n)) { let len = e.lines[p.line.min(e.lines.len() - 1)].chars().count().max(1); e.hover = Some(vc_gui_editor::core::HoverInfo { von: Pos { line: p.line, col: 0 }, bis: Pos { line: p.line, col: len }, text: format!("**{}**\n{}", if d.fehler { "Fehler" } else if d.hinweis { "Hinweis" } else { "Warnung" }, d.text) }) } }
            if let Some(n) = self.name().filter(|n| vc_ide_model::editor_names::is_native_basic(n)) { self.basic_hover(&n, p) }
            if let Some(n) = self.name().filter(|n| self.lsp_gesendet.contains_key(n)) { self.hover_req += 1; self.lsp.senden(lsp::Cmd::Hover { uri: lsp::uri(&self.root, &n), zeile: p.line, spalte: p.col, req: self.hover_req }) } }
    }

    /// Prettyprint beim Verlassen der Zeile: merkt die erste Zeile einer Bearbeitung und formatiert sie, sobald der Cursor eine andere Zeile erreicht
    /// (nicht bei Auswahl, offener Vervollständigung, Undo/Redo, Dateiwechsel; nur Assembler und BASIC)
    /// Ziehen aus dem Projektbaum in den Quelltext: die passende Include-Zeile (`vc_ide_model::dropinc`) vor die Zeilengrenze `k` einfügen
    fn quelltext_ablegen(&mut self, k: usize) {
        for e in self.editoren.values_mut() { e.einfuegemarke = None }
        let Some(g) = self.baum_zug.take() else { return };
        self.include_ablegen(&g.quelle, Some(k));
    }
    /// Include-Zeile für `quelle` im aktiven Quelltext einfügen: vor die Zeilengrenze `k`, ohne Angabe unter die Cursorzeile.
    /// Schon eingebunden: nichts einfügen, zur vorhandenen Zeile springen. Ablehnungen und Erfolg als Hinweis in der Statusleiste.
    fn include_ablegen(&mut self, quelle: &str, k: Option<usize>) {
        let Some(n) = self.name().filter(|n| self.editoren.contains_key(n)) else { self.hinweis = "Kein Quelltext offen: hier gibt es nichts zum Einbinden".into(); return };
        let text = self.editoren[&n].text();
        match vc_ide_model::dropinc::plan(&n, &text, quelle, Some(&self.root)) {
            Ok(plan) => {
                let Some(ed) = self.editoren.get_mut(&n) else { return };
                let l = ed.lines.len(); let k = k.unwrap_or(ed.cursor.line + 1);
                if k < l { ed.replace(Pos { line: k, col: 0 }, Pos { line: k, col: 0 }, &format!("{}\n", plan.line)) }
                else { let e = Pos { line: l - 1, col: ed.line_len(l - 1) }; ed.replace(e, e, &format!("\n{}", plan.line)) }
                self.hinweis = format!("Eingebunden: {}{} ({}+Z nimmt es zurück)", plan.line, plan.alt.map(|(a, _)| format!(" · Alternative von Hand: {a}")).unwrap_or_default(), befehle::plattform_text("Cmd"));
            }
            Err(vc_ide_model::dropinc::Reject::Already(z)) => {
                if let Some(ed) = self.editoren.get_mut(&n) { let z = z.min(ed.lines.len() - 1); ed.set_cursor(Pos { line: z, col: 0 }, false); ed.center_pending = Some(z) }
                self.hinweis = format!("Diese Datei ist schon eingebunden (Zeile {}) – nichts eingefügt", z + 1);
            }
            Err(e) => self.hinweis = e.to_string(),
        }
    }
    pub(crate) fn status_hinweis(&mut self, t: &str) { self.hinweis = t.to_string() }
    fn pretty_pruefen(&mut self, text_aenderung: bool, rueckgaengig: bool, von_hand: bool, vor: (usize, usize)) {
        let Some(n) = self.name() else { return };
        if rueckgaengig { self.pretty_zeile = None; return }
        if text_aenderung && self.pretty_zeile.as_ref().is_none_or(|(d, ..)| *d != n) { self.pretty_zeile = Some((n.clone(), vor.0, vor.1)) }
        if !von_hand { return } // Zeigerbewegungen, Größenänderung, Suchen und andere Meldungen verlassen keine Zeile „von Hand“
        let Some((d, l, zahl)) = self.pretty_zeile.clone() else { return };
        if d != n { self.pretty_zeile = None; return }
        let an = self.einst.pretty_zeile;
        let Some(ed) = self.editoren.get_mut(&n) else { return };
        if ed.cursor.line == l || ed.completion.is_some() { return }
        self.pretty_zeile = None;
        let z = ed.lines.len();
        if an && ed.selection().is_none() && z >= zahl && z <= zahl + 1 { prettyzeile::zeile_formatieren(ed, &n, l); }
    }

    /// Hover im nativen BASIC: Zahlen, Register und Wertzerlegung aus dem Kern (`vc_ide_model::basichover`), ohne Sprachdienst; die Stelle kommt
    /// von der Maus (`p`), nie vom Textcursor
    fn basic_hover(&mut self, rel: &str, p: Pos) {
        let sys = self.system_id();
        let dialekt = vc_target::load_target(&self.data, &sys).ok().and_then(|t| t.basic_dialect).unwrap_or_else(|| "cbm_v2".into());
        let Some(e) = self.editoren.get_mut(rel) else { return };
        let karte = vc_ide_model::basichover::hover(&self.data, &sys, &dialekt, &e.text(), p.line, p.col);
        e.hover = karte.map(|k| vc_gui_editor::core::HoverInfo { von: Pos { line: p.line, col: k.start }, bis: Pos { line: p.line, col: k.end.max(k.start + 1) }, text: k.md });
    }

    fn befehl(&mut self, c: Cmd) -> Task<Msg> {
        if !self.nutzbar(c) { return Task::none() }
        match c {
            Cmd::Palette => { self.palette.offen = !self.palette.offen; self.palette.frage.clear(); self.palette.auswahl = 0; if self.palette.offen { return self.text_fokus("palette") } }
            Cmd::Bauen => return self.bauen(Danach::Nichts),
            Cmd::Starten => return self.bauen(Danach::Start),
            Cmd::BauenAktiv => return self.bauen_wahl(Danach::Nichts, true),
            Cmd::StartenAktiv => return self.bauen_wahl(Danach::Start, true),
            Cmd::DebuggenAktiv => { if self.dbg.is_some() { self.rueckfrage = Some(Rueckfrage::DebugNeu); return Task::none() } return self.bauen_wahl(Danach::Debug, true) }
            Cmd::Debuggen => { if self.dbg.is_some() { self.rueckfrage = Some(Rueckfrage::DebugNeu); return Task::none() } return self.bauen(Danach::Debug) }
            Cmd::DebugWeiter => { if let Some(d) = &self.dbg { d.send(debug::Cmd::Cont) } }
            Cmd::DebugPause => { if let Some(d) = &self.dbg { d.send(debug::Cmd::Pause) } }
            Cmd::DebugUeber => { if let Some(d) = &self.dbg { d.send(debug::Cmd::Step("over")) } }
            Cmd::DebugHinein => { if let Some(d) = &self.dbg { d.send(debug::Cmd::Step("into")) } }
            Cmd::DebugRaus => { if let Some(d) = &self.dbg { d.send(debug::Cmd::Step("out")) } }
            Cmd::DebugEnde => self.debug_ende(),
            Cmd::Exportieren => { self.export_oeffnen() }
            Cmd::TestsAusfuehren => return self.tests_starten(vc_ide::tests::suite::Only::All),
            Cmd::DebugHierher => {
                let (Some(n), Some(info)) = (self.name(), self.info.clone()) else { return Task::none() };
                let zeile = self.editoren.get(&n).map(|e| e.cursor.line + 1).unwrap_or(1);
                match info.adresse_von(&n, zeile) { None => self.log.push(format!("Zeile {zeile} hat keinen Code: dort kann nicht angehalten werden.")), Some(a) => if let Some(d) = &self.dbg { d.send(debug::Cmd::AddPoint { kind: "exec".into(), start: a, end: a, cond: None, ignore: 0 }); d.send(debug::Cmd::Cont); self.run_to = Some((a, false)) } }
            }
            Cmd::Speichern => self.speichern(),
            Cmd::StudioVergroessern | Cmd::StudioVerkleinern | Cmd::StudioEinpassen | Cmd::StudioRaster | Cmd::StudioZwiebel => { if let Some(n) = self.name() {
                let (plus, minus, fit) = (c == Cmd::StudioVergroessern, c == Cmd::StudioVerkleinern, c == Cmd::StudioEinpassen);
                if self.studios.contains_key(&n) { self.studio_aktion(&n, if plus { studio::Aktion::ZoomPlus } else if minus { studio::Aktion::ZoomMinus } else if fit { studio::Aktion::ZoomAnpassen } else if c == Cmd::StudioRaster { studio::Aktion::Raster } else { studio::Aktion::Zwiebel }) }
                else if self.screens.contains_key(&n) && (plus || minus || fit) { self.screen_aktion(&n, if plus { screenstudio::SAkt::Zoom(1) } else if minus { screenstudio::SAkt::Zoom(-1) } else { screenstudio::SAkt::Einpassen }) }
                else if self.tilesets.contains_key(&n) && (plus || minus || fit) { self.tile_aktion(&n, if plus { tilestudio::TAkt::Zoom(1) } else if minus { tilestudio::TAkt::Zoom(-1) } else { tilestudio::TAkt::Einpassen }) }
                else if self.banken.contains_key(&n) { let a = if plus { Some(bankstudio::BAkt::Zoom(1)) } else if minus { Some(bankstudio::BAkt::Zoom(-1)) } else if fit { Some(bankstudio::BAkt::Einpassen) } else if c == Cmd::StudioRaster { self.banken.get(&n).map(|b| bankstudio::BAkt::Gitter(!b.zellgitter)) } else { None }; if let Some(a) = a { self.bank_aktion(&n, a) } } } }
            Cmd::TrackerNoten | Cmd::TrackerInstrumente | Cmd::TrackerAblauf => { if let Some(n) = self.name().filter(|n| self.songs.contains_key(n)) { let m = match c { Cmd::TrackerNoten => songstudio::SongModus::Noten, Cmd::TrackerInstrumente => songstudio::SongModus::Instrumente, _ => songstudio::SongModus::Ablauf }; self.song_aktion(&n, songstudio::GAkt::Modus(m)) } }
            Cmd::AlleSpeichern => { for n in self.alle_namen() { if !self.ist_scratch(&n) { self.speichere_datei(&n) } } }
            Cmd::NeuerReiter | Cmd::NeueNotiz => self.scratch_neu(c == Cmd::NeueNotiz, String::new()),
            Cmd::DateiOeffnen => return Task::perform(async { rfd::AsyncFileDialog::new().set_title("Datei öffnen").pick_file().await.map(|f| f.path().to_path_buf()) }, Msg::Geoeffnet),
            // Projektdatei wählen (ein Klick wählt, Doppelklick oder „Öffnen“ öffnet); in einem Ordner-Dialog sind Dateien ausgegraut und lassen sich nicht anklicken
            Cmd::ProjektOeffnen => return Task::perform(async { rfd::AsyncFileDialog::new().set_title("Projekt öffnen (Projektdatei .vcprj)").add_filter("VintageCoder-Projekt", &["vcprj"]).pick_file().await.map(|f| f.path().to_path_buf()) }, Msg::Geoeffnet),
            Cmd::ProjektOrdnerOeffnen => return Task::perform(async { rfd::AsyncFileDialog::new().set_title("Projektordner öffnen").pick_folder().await.map(|f| f.path().to_path_buf()) }, Msg::Geoeffnet),
            Cmd::DateiSchliessen => { let i = self.aktiv; return self.update_innen(Msg::Schliesse(i)) }
            Cmd::Rueckgaengig => { match self.name().filter(|n| self.studios.contains_key(n) || self.paletten.contains_key(n) || self.screens.contains_key(n) || self.tilesets.contains_key(n) || self.banken.contains_key(n) || self.songs.contains_key(n)) { Some(n) if self.songs.contains_key(&n) => self.song_aktion(&n, songstudio::GAkt::Undo), Some(n) if self.banken.contains_key(&n) => self.bank_aktion(&n, bankstudio::BAkt::Undo), Some(n) if self.tilesets.contains_key(&n) => self.tile_aktion(&n, tilestudio::TAkt::Undo), Some(n) if self.screens.contains_key(&n) => self.screen_aktion(&n, screenstudio::SAkt::Undo), Some(n) if self.paletten.contains_key(&n) => self.palette_aktion(&n, palettestudio::PAkt::Undo), Some(n) => self.studio_aktion(&n, studio::Aktion::Undo), None => self.ed_nachricht(EdMsg::Undo) } } Cmd::Wiederholen => { match self.name().filter(|n| self.studios.contains_key(n) || self.paletten.contains_key(n) || self.screens.contains_key(n) || self.tilesets.contains_key(n) || self.banken.contains_key(n) || self.songs.contains_key(n)) { Some(n) if self.songs.contains_key(&n) => self.song_aktion(&n, songstudio::GAkt::Redo), Some(n) if self.banken.contains_key(&n) => self.bank_aktion(&n, bankstudio::BAkt::Redo), Some(n) if self.tilesets.contains_key(&n) => self.tile_aktion(&n, tilestudio::TAkt::Redo), Some(n) if self.screens.contains_key(&n) => self.screen_aktion(&n, screenstudio::SAkt::Redo), Some(n) if self.paletten.contains_key(&n) => self.palette_aktion(&n, palettestudio::PAkt::Redo), Some(n) => self.studio_aktion(&n, studio::Aktion::Redo), None => self.ed_nachricht(EdMsg::Redo) } }
            Cmd::Suchen | Cmd::Ersetzen => { self.find.offen = true; self.find.ersetzen = c == Cmd::Ersetzen; self.goto.offen = false; let sel = self.ed().and_then(|e| e.selected_text()).filter(|s| !s.contains('\n')); if let Some(s) = sel { self.find.frage = s } self.find_aktualisieren(true); return self.text_fokus("find") }
            Cmd::GeheZuZeile => { self.goto.offen = true; self.find.offen = false; self.goto.text.clear(); return self.text_fokus("goto") }
            Cmd::InProjektSuchen => { self.seite = Seite::Suche; self.layout.aufklappen(Bereich::Seitenleiste); self.layout_merken(); return self.text_fokus("suche") }
            Cmd::MdFett | Cmd::MdKursiv | Cmd::MdCode | Cmd::MdDurchgestrichen | Cmd::MdLink | Cmd::MdBild | Cmd::MdUeberschriftTiefer | Cmd::MdUeberschriftHoeher | Cmd::MdAufzaehlung | Cmd::MdNummerierung | Cmd::MdAufgaben | Cmd::MdZitat | Cmd::MdAufgabeUmschalten | Cmd::MdTabelleNeu | Cmd::MdTabelleAusrichten | Cmd::MdAlsMarkdown => return self.md_befehl(c),
            Cmd::MdUmschalten => { if let Some(n) = self.name().filter(|n| (n.to_lowercase().ends_with(".md") || self.md_als.contains(n)) && self.editoren.contains_key(n)) { return self.update_innen(Msg::MdUmschalten(n)) } }
            Cmd::Zahlenwerkzeuge => { self.zahlen = Some(zahlen::ZahlenDlg::default()); return self.text_fokus("zahl") }
            Cmd::BasicNachAsm => self.basic_nach_asm(),
            Cmd::Zen => { self.layout.zen_umschalten(); self.layout_merken() }
            Cmd::Seitenleiste => { self.layout.umschalten(Bereich::Seitenleiste); self.layout_merken() }
            Cmd::Schublade => { self.layout.umschalten(Bereich::Schublade); self.layout_merken() }
            Cmd::MaximiereFokus => { let b = self.fokus_bereich; self.layout.umschalten_maximiert(b); self.layout_merken() }
            Cmd::SpalteMax => { self.layout.umschalten_maximiert(Bereich::Seitenleiste); self.layout_merken() }
            Cmd::DokumentMax => { self.layout.umschalten_maximiert(Bereich::Dokument); self.layout_merken() }
            Cmd::SchubladeMax => { self.layout.umschalten_maximiert(Bereich::Schublade); self.layout_merken() }
            Cmd::PanelMax => { self.layout.umschalten_maximiert(Bereich::Panel); self.layout_merken() }
            Cmd::AlsInclude => { if let Some(q) = self.ziel.take() { self.include_ablegen(&q, None) } }
            Cmd::ZeileFormatieren => { if let Some(n) = self.name() { let l = self.editoren.get(&n).map(|e| e.cursor.line); if let (Some(l), Some(ed)) = (l, self.editoren.get_mut(&n)) { if !prettyzeile::zeile_formatieren(ed, &n, l) { self.status_hinweis("Zeile ist schon formatiert (oder hier gibt es keine Formatierung)") } } } }
            Cmd::DateiFormatieren => { if let Some(n) = self.name() { if let Some(ed) = self.editoren.get_mut(&n) { if !prettyzeile::datei_formatieren(ed, &n) { self.status_hinweis("Datei ist schon formatiert (oder hier gibt es keine Formatierung)") } } } }
            Cmd::EditorSchriftGroesser => return self.update_innen(Msg::SetSchrift(self.einst.editor_schrift + 1.0)),
            Cmd::EditorSchriftKleiner => return self.update_innen(Msg::SetSchrift(self.einst.editor_schrift - 1.0)),
            Cmd::DarstellungZuruecksetzen => { self.layout.zuruecksetzen(); self.layout_merken(); let _ = self.update_innen(Msg::SetZoom(1.0)); return self.update_innen(Msg::SetSchrift(Einstellungen::default().editor_schrift)) }
            Cmd::LayoutZuruecksetzen => { self.layout.zuruecksetzen(); self.layout_merken() }
            Cmd::Panel => { if self.panel_offen() { self.layout.minimieren(Bereich::Panel); self.layout_merken() } else { let p = self.panel_seite; return self.panel_oeffnen(p) } }
            Cmd::ZoomPlus => return self.update_innen(Msg::SetZoom(self.einst.zoom + 0.1)), Cmd::ZoomMinus => return self.update_innen(Msg::SetZoom(self.einst.zoom - 0.1)), Cmd::ZoomNull => return self.update_innen(Msg::SetZoom(1.0)),
            Cmd::HoherKontrast => { let s = if self.einst.schema == Schema::Kontrast { Schema::System } else { Schema::Kontrast }; return self.update_innen(Msg::SetSchema(s)) }
            Cmd::HellDunkel => { let s = if self.dunkel() { Schema::Hell } else { Schema::Dunkel }; return self.update_innen(Msg::SetSchema(s)) }
            Cmd::Umbruch => { let u = !self.einst.umbruch; return self.update_innen(Msg::SetUmbruch(u)) }
            Cmd::Einstellungen => self.tab_oeffnen(Tab::Einstellungen),
            Cmd::Zeilenende | Cmd::Kodierung => { if let Some(n) = self.name().filter(|n| self.editoren.contains_key(n)) { let mut mt = self.textmeta.get(&n).copied().unwrap_or_default(); if c == Cmd::Zeilenende { mt.crlf = !mt.crlf } else { mt.latin1 = !mt.latin1; mt.bom = false } self.textmeta.insert(n.clone(), mt); if let Some(e) = self.editoren.get_mut(&n) { e.dirty = true } self.log.push(format!("{n}: {} · {} (gilt beim Speichern)", mt.kodierung(), mt.zeilenende())) } }
            Cmd::ProjektNeu => self.neuprojekt_oeffnen(),
            Cmd::Assistent => self.wiz_oeffnen(None),
            Cmd::AssistentRoms => self.wiz_oeffnen(Some(vc_ide::setup::Mode::Part(vc_ide::setup::Part::Roms))),
            Cmd::AssistentEmulatoren => self.wiz_oeffnen(Some(vc_ide::setup::Mode::Part(vc_ide::setup::Part::Emulators))),
            Cmd::AssistentHardware => self.wiz_oeffnen(Some(vc_ide::setup::Mode::Part(vc_ide::setup::Part::Hardware))),
            Cmd::AssistentDesign => self.wiz_oeffnen(Some(vc_ide::setup::Mode::Part(vc_ide::setup::Part::DesignEditor))),
            // „Nach Neuem suchen“: nur auf Knopfdruck (nie von selbst): ROM-Verzeichnis neu einlesen und die Emulatorsuche des Assistenten starten
            Cmd::AssistentNeues => { vc_media::romref::rescan_async(); self.log.push("Nach Neuem gesucht: ROM-Verzeichnis neu eingelesen, Emulatorsuche im Assistenten.".into()); self.wiz_oeffnen(Some(vc_ide::setup::Mode::Part(vc_ide::setup::Part::Emulators))) }
            Cmd::Ueber => return self.info_akt(infodlg::InfoAkt::UeberAuf),
            Cmd::Feedback => return self.info_akt(infodlg::InfoAkt::FeedbackAuf),
            Cmd::UpdatePruefen => return self.info_akt(infodlg::InfoAkt::Pruefen),
            Cmd::ProjektKopieren => self.kopieren_oeffnen(),
            // Art des Reiters (nur Darstellung und Sprachdienste, die Datei bleibt unverändert)
            Cmd::ArtText | Cmd::ArtAsm | Cmd::ArtBasic => { let l = match c { Cmd::ArtAsm => vc_ide_model::lang::Lang::Asm, Cmd::ArtBasic => vc_ide_model::lang::Lang::Basic, _ => vc_ide_model::lang::Lang::Plain }; if let Some(e) = self.ed() { e.lang = l } }
            Cmd::HilfeImReiter => return self.update_innen(Msg::HilfeImReiter),
            Cmd::HexVorschlaege => self.hexv_oeffnen(),
            Cmd::StdZeichensatz => self.zeichen_oeffnen(zeichendlg::ZTab::Standard),
            Cmd::ZeichensatzAusRom => self.zeichen_oeffnen(zeichendlg::ZTab::Rom),
            Cmd::ZeichensatzAusPng => self.zeichen_oeffnen(zeichendlg::ZTab::Png),
            Cmd::ProjektEinstellungen => self.pe_oeffnen(),
            Cmd::Beenden => { let l = self.ungespeichert(); if l.is_empty() { return self.beenden() } self.beenden_frage = Some((l, Folge::Beenden)) }
            Cmd::Startseite => self.tab_oeffnen(Tab::Start),
            Cmd::NaechstesProblem | Cmd::VorigesProblem => { if !self.probleme.is_empty() { let fehler: Vec<usize> = (0..self.probleme.len()).collect(); let cur = self.probleme.iter().position(|p| Some(&p.datei) == self.name().as_ref() && self.editoren.get(&p.datei).map_or(false, |e| e.cursor.line + 1 == p.zeile)); let n = fehler.len() as i32; let i = match cur { Some(i) => (i as i32 + if c == Cmd::NaechstesProblem { 1 } else { -1 }).rem_euclid(n) as usize, None => 0 }; self.schublade = 1; return self.update_innen(Msg::GotoProblem(i)) } }
            Cmd::Teilen => self.teilen_umschalten(),
            Cmd::ReiterLinks => self.reiter_verschieben(-1),
            Cmd::ReiterRechts => self.reiter_verschieben(1),
            Cmd::NaechsterReiter => { if !self.tabs.is_empty() { self.aktiv = (self.aktiv + 1) % self.tabs.len(); self.markiere() } }
            Cmd::VorigerReiter => { if !self.tabs.is_empty() { self.aktiv = (self.aktiv + self.tabs.len() - 1) % self.tabs.len(); self.markiere() } }
            Cmd::Hilfe => { let wort = self.wort_unter_cursor(); let t = self.panel_oeffnen(PanelSeite::Hilfe); if let Some(w) = wort { self.hilfe.suche = w.clone(); self.hilfe.treffer = vc_ide_model::helpstate::search(&self.data, &self.hilfe_ziel(), &w); if let Some(d) = vc_ide_model::helpstate::open(&self.data, &self.hilfe_ziel(), self.hilfe_sprache(), &w) { self.hilfe.doc = Some(d) } } return t }
            Cmd::NeueDatei => { let o = self.ziel.take().unwrap_or_default(); let o = self.ordner_zu(o); return self.update_innen(Msg::BaumNeu(o, NeuArt::Asm)) }
            Cmd::NeueNotizDialog => { let o = self.ziel.take().unwrap_or_default(); let o = self.ordner_zu(o); return self.update_innen(Msg::BaumNeu(o, NeuArt::Notiz)) }
            Cmd::NeuerOrdner => { let o = self.ziel.take().unwrap_or_default(); let o = self.ordner_zu(o); return self.update_innen(Msg::BaumOrdner(o)) }
            Cmd::ZielOeffnen => { if let Some(rel) = self.ziel.take().or_else(|| self.name()) { self.oeffne(&rel) } }
            Cmd::DateiUmbenennen => return self.update_innen(Msg::BaumUmbenennen),
            Cmd::DateiLoeschen => return self.update_innen(Msg::BaumLoeschen),
            Cmd::AlleAuswaehlen => { if let Some(n) = self.name().filter(|n| self.studios.contains_key(n)) { self.studio_aktion(&n, studio::Aktion::AuswahlAlles) } else { self.ed_nachricht(EdMsg::SelectAll) } }
            Cmd::AllesFalten => { if let Some(ed) = self.ed() { let n = ed.lines.len(); for l in 0..n { if !ed.is_hidden(l) && !ed.folds.contains_key(&l) && ed.fold_range(l).is_some() { ed.toggle_fold(l) } } } }
            Cmd::AllesEntfalten => { if let Some(ed) = self.ed() { let starts: Vec<usize> = ed.folds.keys().copied().collect(); for l in starts { ed.toggle_fold(l) } } }
            Cmd::Vervollstaendigen => { let vokabular: Vec<&str> = vc_gui_editor::highlight::MNEMONICS.iter().chain(vc_gui_editor::highlight::DIRECTIVES.iter()).copied().collect(); if let Some(ed) = self.ed() { ed.update_completion_ab(&vokabular, 0); if ed.completion.is_none() { self.hinweis = "Keine Vorschläge an dieser Stelle".into() } } }
            Cmd::ProblemSpringen => { let i = self.ziel.take().and_then(|z| z.parse::<usize>().ok()).unwrap_or(0); return self.update_innen(Msg::GotoProblem(i)) }
            Cmd::ProblemKopieren => { let i = self.ziel.take().and_then(|z| z.parse::<usize>().ok()).unwrap_or(0); if let Some(p) = self.probleme.get(i) { return iced::clipboard::write(format!("{}:{}: {}", p.datei, p.zeile, p.text)) } else { self.hinweis = "Keine Probleme".into() } }
            Cmd::ProblemeAlleKopieren => { if self.probleme.is_empty() { self.hinweis = "Keine Probleme".into() } else { return iced::clipboard::write(self.probleme.iter().map(|p| format!("{}:{}: {}", p.datei, p.zeile, p.text)).collect::<Vec<_>>().join("\n")) } }
            Cmd::ProtokollKopieren => { return iced::clipboard::write(self.log.join("\n")) }
            Cmd::ProtokollLeeren => { self.log.clear() }
            Cmd::PfadKopieren => { let rel = self.ziel.take().or_else(|| self.name()); if let Some(rel) = rel { let p = project::pfad(&self.root, &rel).to_string_lossy().into_owned(); self.log.push(format!("Pfad kopiert: {p}")); return iced::clipboard::write(p) } }
            Cmd::BaumRueckgaengig => self.baum_rueckgaengig(),
            Cmd::HexAnsicht => { let rel = self.ziel.take().or_else(|| self.name()); if let Some(rel) = rel { if project::pfad(&self.root, &rel).is_file() { self.hex_oeffnen(&rel) } else { self.log.push(t::BAUM_KEINE_DATEI.into()) } } }
            Cmd::ImOrdnerZeigen => { let rel = self.ziel.take().or_else(|| self.name()); if let Some(rel) = rel { zeige_im_ordner(&project::pfad(&self.root, &rel)) } }
            Cmd::Kopieren | Cmd::Ausschneiden | Cmd::Einfuegen if self.name().is_some_and(|n| self.songs.contains_key(&n)) => { let n = self.name().unwrap_or_default(); self.song_aktion(&n, match c { Cmd::Kopieren => songstudio::GAkt::Kopieren, Cmd::Ausschneiden => songstudio::GAkt::Ausschneiden, _ => songstudio::GAkt::Einfuegen }) }
            Cmd::Kopieren | Cmd::Ausschneiden if self.name().is_some_and(|n| self.studios.contains_key(&n)) => { let n = self.name().unwrap_or_default(); self.studio_aktion(&n, if c == Cmd::Kopieren { studio::Aktion::Kopieren } else { studio::Aktion::Ausschneiden }) }
            Cmd::Einfuegen if self.name().is_some_and(|n| self.studios.contains_key(&n)) => { let n = self.name().unwrap_or_default(); self.studio_aktion(&n, studio::Aktion::Einfuegen) }
            Cmd::Kopieren | Cmd::Ausschneiden => { let sel = self.ed().and_then(|e| e.selected_text()); if let Some(t) = sel { if c == Cmd::Ausschneiden { if let Some(e) = self.ed() { e.insert("") } } self.editor_fokus(); return iced::clipboard::write(t) } }
            Cmd::Einfuegen => return iced::clipboard::read().map(Msg::Einfuegen),
            Cmd::GeheZuDefinition => { if let Some(n) = self.name().filter(|n| self.lsp_gesendet.contains_key(n)) { if let Some(e) = self.editoren.get(&n) { self.def_req += 1; self.lsp.senden(lsp::Cmd::Definition { uri: lsp::uri(&self.root, &n), zeile: e.cursor.line, spalte: e.cursor.col, req: self.def_req }) } } else { self.log.push("Gehe zu Definition: nur in Assembler-Dateien.".into()) } }
            Cmd::HaltepunktUmschalten => { if let Some(e) = self.ed() { let l = e.cursor.line; if !e.breakpoints.remove(&l) { e.breakpoints.insert(l); } } }
            Cmd::ReiterAndereHaelfte => { if let Some(tab) = self.tabs.get(self.aktiv).cloned() { let ziel = if self.andere.is_none() { true } else { !self.fokus_rechts }; self.tab_verschieben(tab, ziel, None) } }
        }
        Task::none()
    }
    // ------------------------------------------------------------ Beenden und Projektwechsel mit ungespeicherten Änderungen
    fn alle_namen(&self) -> Vec<String> { self.editoren.keys().chain(self.studios.keys()).chain(self.paletten.keys()).chain(self.disks.keys()).chain(self.hexe.keys()).chain(self.screens.keys()).chain(self.tilesets.keys()).chain(self.banken.keys()).chain(self.songs.keys()).cloned().collect() }
    fn ist_geaendert(&self, n: &str) -> bool {
        if self.ist_scratch(n) { return self.editoren.get(n).is_some_and(|e| !e.text().trim().is_empty()) }
        self.editoren.get(n).is_some_and(|e| e.dirty && !self.lesefehler.contains(n)) || self.studios.get(n).is_some_and(|s| s.doc.dirty()) || self.paletten.get(n).is_some_and(|p| p.dirty()) || self.disks.get(n).is_some_and(|d| d.dirty) || self.screens.get(n).is_some_and(|s| s.doc.dirty()) || self.tilesets.get(n).is_some_and(|s| s.doc.dirty()) || self.banken.get(n).is_some_and(|s| s.doc.dirty()) || self.songs.get(n).is_some_and(|s| s.doc.dirty())
    }
    /// Namen aller Dateien, Studios, Disketten und unbenannten Reiter mit ungespeicherten Änderungen
    pub(crate) fn ungespeichert(&self) -> Vec<String> { let mut v: Vec<String> = self.alle_namen().into_iter().filter(|n| self.ist_geaendert(n)).collect(); if self.pe.as_ref().is_some_and(|z| z.geaendert()) { v.push("Projekt-Einstellungen".into()) } v.sort(); v.dedup(); v }
    /// Würde das Öffnen von `p` das Projekt wechseln (und damit ungespeicherte Dateien betreffen)?
    fn wechsel_noetig(&self, p: &std::path::Path) -> bool { match project::ziel_fuer(p) { Some(z) => !(z.root == self.root && (z.vcprj == self.proj.file || z.vcprj.is_none())), None => false } }
    /// Programm beenden; im Testlauf nur aufzeichnen (Attrappe in `aussen`)
    fn beenden(&mut self) -> Task<Msg> {
        self.scratch_sichern(); self.marken_sichern();
        if aussen::gesperrt() { aussen::aufzeichnen(aussen::Aufruf::Beenden); return Task::none() }
        iced::exit()
    }
    fn beenden_antwort(&mut self, w: u8) -> Task<Msg> {
        let Some((_, folge)) = self.beenden_frage.take() else { return Task::none() };
        match w {
            // 1 speichern, 2 verwerfen, sonst abbrechen
            1 | 2 => {
                if w == 1 { for n in self.alle_namen() { if self.ist_geaendert(&n) && !self.ist_scratch(&n) { self.speichere_datei(&n) } } if self.pe.as_ref().is_some_and(|z| z.geaendert()) { let _ = self.pe_akt(projeinst::PeAkt::Speichern(false)); } }
                else { for (n, _) in self.scratch.clone() { self.scratch_verwerfen(&n) } self.scratch_kept.clear(); self.scratch_stand.clear(); self.scratch_sichern() }
                // Notizen mit Inhalt bleiben bei „Speichern“ in der Zwischenkopie (Wiederherstellung beim nächsten Start)
                match folge { Folge::Beenden => return self.beenden(), Folge::Projekt(p) => { self.wechsel_ohne_speichern = true; self.projekt_oeffnen(p); self.wechsel_ohne_speichern = false } }
            }
            _ => {}
        }
        Task::none()
    }
    fn beenden_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let (liste, folge) = self.beenden_frage.as_ref()?;
        let frage = match folge { Folge::Beenden => "VintageCoder beenden?", Folge::Projekt(_) => "Projekt wechseln?" };
        let mut c = column![text(frage).size(16).color(pal.text), text("Diese Reiter haben ungespeicherte Änderungen:").size(12.5).color(pal.mute)].spacing(10);
        for n in liste { c = c.push(text(format!("• {n}{}", if self.ist_scratch(n) { "  (unbenannt, wird in der Zwischenkopie gesichert)" } else { "" })).size(12.5).color(pal.text)) }
        c = c.push(row![knopf(pal, "Speichern", Some(Msg::BeendenAntwort(1)), pal.text, true), knopf(pal, "Verwerfen", Some(Msg::BeendenAntwort(2)), pal.text, false), knopf(pal, "Abbrechen", Some(Msg::BeendenAntwort(0)), pal.text, false)].spacing(8).wrap().vertical_spacing(6));
        let karte = container(c).width(480).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hintergrund = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() })).on_press(Msg::BeendenAntwort(0));
        Some(stack![hintergrund, container(karte).center(Length::Fill)].into())
    }
    // ------------------------------------------------------------ Unbenannte Reiter und Notizen
    fn ist_scratch(&self, n: &str) -> bool { self.scratch.iter().any(|(s, _)| s == n) }
    /// Neuer unbenannter Reiter bzw. neue Notiz: nur im Speicher, nie als Datei im Projekt, bis „Speichern unter“
    fn scratch_neu(&mut self, notiz: bool, text: String) {
        self.scratch_n += 1; let n = format!("{} {}", if notiz { "Notiz" } else { "Unbenannt" }, self.scratch_n);
        let mut e = Editor::new(&text); e.lang = vc_ide_model::lang::Lang::of(if notiz { "x.md" } else { "x.asm" }); e.wrap = self.einst.umbruch; e.dirty = !text.is_empty();
        self.editoren.insert(n.clone(), e); self.scratch.push((n.clone(), notiz)); self.tab_oeffnen(Tab::Datei(n)); self.editor_fokus();
    }
    fn scratch_verwerfen(&mut self, n: &str) {
        self.scratch.retain(|(s, _)| s != n); self.editoren.remove(n); self.marken_sichern();
        if let Some(i) = self.tabs.iter().position(|t| matches!(t, Tab::Datei(x) if x == n)) { self.tabs.remove(i); self.aktiv = self.aktiv.min(self.tabs.len().saturating_sub(1)); if self.tabs.is_empty() { if let Some((t, a)) = self.andere.take() { self.tabs = t; self.aktiv = a; self.fokus_rechts = false } } self.markiere() }
        self.scratch_stand.clear(); self.scratch_sichern();
    }
    fn speichern_unter_dialog(&mut self, n: &str) {
        let notiz = self.scratch.iter().any(|(s, nt)| s == n && *nt);
        self.baum = Some(BaumDialog { art: BaumArt::SpeichernUnter(n.to_string()), text: if notiz { "notiz.md".into() } else { "neu.asm".into() }, fehler: String::new() });
    }
    /// Unbenannten Reiter als Datei im Projekt ablegen; Name ohne Endung bekommt `.asm` bzw. `.md`
    fn scratch_speichern_unter(&mut self, n: &str, rel: &str) -> Result<(), String> {
        let notiz = self.scratch.iter().any(|(s, nt)| s == n && *nt);
        let rel = if rel.contains('.') { rel.to_string() } else { format!("{rel}.{}", if notiz { "md" } else { "asm" }) };
        let text = self.editoren.get(n).map(|e| e.text()).ok_or("Reiter nicht gefunden")?;
        let rel = vc_ide_model::fileops::create_file(&self.root, &rel)?;
        textdatei::schreiben(&self.root.join(&rel), &text, &textdatei::Meta::default()).map_err(|e| { let _ = std::fs::remove_file(self.root.join(&rel)); e })?;
        self.scratch.retain(|(s, _)| s != n);
        if let Some(mut e) = self.editoren.remove(n) { e.lang = vc_ide_model::lang::Lang::of(&rel); e.dirty = false; self.editoren.insert(rel.clone(), e); }
        for t in self.tabs.iter_mut().chain(self.andere.iter_mut().flat_map(|(t, _)| t.iter_mut())) { if matches!(t, Tab::Datei(x) if x == n) { *t = Tab::Datei(rel.clone()) } }
        self.projekt_neu_laden(); self.log.push(format!("Gespeichert: {rel}")); self.scratch_stand.clear(); self.scratch_sichern(); self.markiere();
        if rel.ends_with(".asm") || rel.ends_with(".inc") { let e = &self.editoren[&rel]; self.lsp.senden(lsp::Cmd::Open { uri: lsp::uri(&self.root, &rel), text: e.text() }); self.lsp_gesendet.insert(rel.clone(), e.version); }
        Ok(())
    }
    /// Zwischenkopie der unbenannten Reiter schreiben (nur wenn sich etwas geändert hat; eigene Datei im Konfigurationsordner, nie im Projekt)
    pub(crate) fn scratch_sichern(&mut self) {
        let Some(f) = self.scratch_datei.clone() else { return }; if !vc_ide_model::scratchsave::enabled() && !cfg!(test) { return }
        let jetzt = vc_ide_model::scratchsave::now_secs();
        let mut eintraege: Vec<vc_ide_model::scratchsave::Entry> = self.scratch_kept.clone(); eintraege.extend(self.scratch_rest.clone().unwrap_or_default());
        eintraege.extend(self.scratch.iter().filter_map(|(n, nt)| self.editoren.get(n).map(|e| (n, *nt, e.text()))).filter(|(_, _, t)| !t.trim().is_empty()).map(|(n, nt, t)| vc_ide_model::scratchsave::Entry { name: n.clone(), text: t, note: nt, saved_at: jetzt }));
        let schluessel = serde_json::to_string(&eintraege.iter().map(|e| (&e.name, &e.text)).collect::<Vec<_>>()).unwrap_or_default();
        if schluessel == self.scratch_stand { return } if vc_ide_model::scratchsave::save_to(&f, &eintraege) { self.scratch_stand = schluessel }
    }
    /// Antwort auf „Letzte Sitzung wiederherstellen?“: 1 wiederherstellen, 2 verwerfen, 0 nicht jetzt (die Sicherung bleibt)
    fn scratch_wiederherstellen(&mut self, w: u8) {
        let Some(e) = self.scratch_rest.take() else { return };
        match w {
            1 => { self.scratch_kept.clear(); for x in e { self.scratch_neu(x.note, x.text) } self.scratch_stand.clear(); self.scratch_sichern() }
            2 => { self.scratch_kept.clear(); self.scratch_stand.clear(); self.scratch_sichern(); if let Some(f) = self.scratch_datei.clone() { if self.scratch.is_empty() { let _ = std::fs::remove_file(f); } } }
            _ => { self.scratch_kept = e }
        }
    }
    fn scratch_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let e = self.scratch_rest.as_ref()?;
        let jetzt = vc_ide_model::scratchsave::now_secs();
        let mut c = column![text("Letzte Sitzung wiederherstellen?").size(16).color(pal.text), text("Beim letzten Beenden waren unbenannte Reiter ungespeichert. Die Zwischenkopie liegt nur auf diesem Rechner, unverschlüsselt, und verfällt nach 14 Tagen.").size(12.5).color(pal.mute)].spacing(10);
        for x in e { c = c.push(text(format!("• {} · {} Zeichen · {}", x.name, x.text.chars().count(), vc_ide_model::recent::ago(x.saved_at, jetzt))).size(12.5).color(pal.text)) }
        c = c.push(row![knopf(pal, "Wiederherstellen", Some(Msg::ScratchRest(1)), pal.text, true), knopf(pal, "Verwerfen", Some(Msg::ScratchRest(2)), pal.text, false), knopf(pal, "Nicht jetzt", Some(Msg::ScratchRest(0)), pal.text, false)].spacing(8).wrap().vertical_spacing(6));
        let karte = container(c).width(480).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        Some(stack![container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() }), container(karte).center(Length::Fill)].into())
    }
    fn speichere(&mut self, n: &str) { if self.ist_scratch(n) { self.speichern_unter_dialog(n); return } self.speichere_datei(n) }
    fn speichere_datei(&mut self, n: &str) { if let Some(s) = self.songs.get_mut(n) { if s.doc.dirty() { match s.doc.speichern() { Ok(()) => self.log.push(format!("Gespeichert: {n}")), Err(e) => self.log.push(e) } } return } if let Some(s) = self.banken.get_mut(n) { if s.doc.dirty() { match s.doc.speichern() { Ok(()) => self.log.push(format!("Gespeichert: {n}")), Err(e) => self.log.push(e) } } return } if let Some(s) = self.tilesets.get_mut(n) { if s.doc.dirty() { match s.doc.speichern() { Ok(()) => self.log.push(format!("Gespeichert: {n}")), Err(e) => self.log.push(e) } } return } if let Some(s) = self.screens.get_mut(n) { if s.doc.dirty() { match s.doc.save() { Ok(()) => self.log.push(format!("Gespeichert: {n}")), Err(e) => self.log.push(e) } } return } if let Some(d) = self.disks.get_mut(n) { if d.dirty && d.path.is_some() { d.speichern(); if let Some((_, m)) = &d.note { self.log.push(m.clone()) } } return } if let Some(p) = self.paletten.get_mut(n) { if p.dirty() { match p.save() { Ok(()) => self.log.push(format!("Gespeichert: {n}")), Err(e) => self.log.push(e) } } return } if let Some(s) = self.studios.get_mut(n) { if s.doc.dirty() { match s.doc.save() { Ok(()) => self.log.push(format!("Gespeichert: {n}")), Err(e) => self.log.push(e) } } return } if self.lesefehler.contains(n) { self.log.push(format!("{n}: nicht gespeichert, die Datei war nicht lesbar und würde überschrieben")); return } let meta = self.textmeta.get(n).copied().unwrap_or_default(); if let Some(e) = self.editoren.get_mut(n) { if e.dirty { match textdatei::schreiben(&self.root.join(n), &e.text(), &meta).map_err(|er| std::io::Error::other(er)) { Ok(_) => { e.dirty = false; self.log.push(format!("Gespeichert: {n}")) } Err(er) => self.log.push(format!("Speichern fehlgeschlagen: {er}")) } } } }
    fn speichern(&mut self) { if let Some(n) = self.name() { self.speichere(&n) } }
    fn bauen(&mut self, danach: Danach) -> Task<Msg> { self.bauen_wahl(danach, false) }
    /// `aktiv`: die Quelldatei im aktiven Reiter bauen; sonst das Startprogramm des Projekts (Hauptquelle), ersatzweise die aktive Datei
    fn bauen_wahl(&mut self, danach: Danach, aktiv: bool) -> Task<Msg> {
        if self.baut { return Task::none() }
        self.danach = danach;
        let aktive = self.name().filter(|n| n.ends_with(".asm"));
        let rel = if aktiv { aktive } else { self.proj.pf.as_ref().and_then(|p| p.main.clone()).filter(|m| m.ends_with(".asm") && self.namen.contains(m)).or_else(|| hauptquelle(&self.namen).filter(|_| self.proj.pf.is_some())).or(aktive) };
        let Some(rel) = rel else { self.log.push("Keine Quelldatei zum Bauen.".into()); self.danach = Danach::Nichts; return Task::none() };
        for n in self.editoren.keys().cloned().collect::<Vec<_>>() { self.speichere(&n) }
        self.baut = true; self.schublade = 0; self.log.push(format!("Bauen: {rel} …"));
        let (root, data, pf) = (self.root.clone(), self.data.clone(), self.proj.pf.clone());
        Task::perform(async move { tokio::task::spawn_blocking(move || vc_ide::projfile::build_gui(&vc_ide::build::Ctx { root: &root, data: &data }, pf.as_ref(), &rel)).await.unwrap_or_else(|e| Err(e.to_string())) }, Msg::BauFertig)
    }
    fn bau_ende(&mut self, r: Result<serde_json::Value, String>) -> bool {
        self.probleme.clear();
        match r {
            Err(e) => { self.log.push(format!("Fehler: {e}")); false }
            Ok(v) => {
                // W203 (zu weiter Sprung automatisch verlängert) hat der Nutzer mit `.branchlong auto` selbst bestellt: kein Problem, eine Protokollzeile genügt
                if let Some(d) = v["diagnostics"].as_array() { let lang = d.iter().filter(|x| x["code"] == "W203").count(); if lang > 0 { self.log.push(format!("{lang} Sprünge automatisch verlängert (.branchlong auto)")) } }
                // Lizenzhinweis (L001, Copyleft im Quelltext): steht im Protokoll und als Information unter „Probleme“, ist kein Fehler
                if let Some(d) = v["diagnostics"].as_array() { for x in d.iter().filter(|x| x["severity"] == "info") { if let Some(m) = x["message"].as_str() { self.log.push(m.to_string()) } } }
                if let Some(d) = v["diagnostics"].as_array() { for x in d.iter().filter(|x| x["code"] != "W203") { self.probleme.push(Problem { datei: x["file"].as_str().unwrap_or("").into(), zeile: x["line"].as_u64().unwrap_or(0) as usize, text: x["message"].as_str().unwrap_or("").into(), fehler: x["severity"] == "error" }) } }
                if v["ok"] == true {
                    self.log.push(format!("Fertig: {} Bytes → {}", v["size"], v["artifact"]));
                    if let Some(d) = v["disk"].as_object() { self.log.push(format!("Disk {} ({} Bytes): {}", d["file"].as_str().unwrap_or(""), d["size"], d["programs"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default())) }
                    if let Some(e) = v["disk_error"].as_str() { self.log.push(format!("Diskette: {e}")); self.schublade_zeigen() } else if let Some(e) = v["disk_skipped"].as_str() { self.log.push(e.to_string()) }
                    let symbols = v["symbols"].as_array().map(|a| a.iter().filter_map(|x| Some((x["name"].as_str()?.to_string(), x["value"].as_i64()?))).collect()).unwrap_or_default();
                    let zeilen = v["lines"].as_array().map(|a| a.iter().filter_map(|l| Some((l["file"].as_str()?.to_string(), l["line"].as_u64()? as usize, l["addr"].as_u64()? as u16, l["len"].as_u64().unwrap_or(1) as usize))).collect()).unwrap_or_default();
                    self.info = Some(BauInfo { target: v["target"].as_str().unwrap_or("c64").into(), artifact: self.root.join(v["artifact"].as_str().unwrap_or("")), symbols, run: v["run"].as_u64().map(|r| r as u16), zeilen, segmente: v["segments"].as_array().map(|a| a.iter().filter_map(|x| Some((x["start"].as_u64()? as u32, x["length"].as_u64()? as usize))).collect()).unwrap_or_default() });
                    self.karte_laden();
                    true
                } else { self.log.push(format!("Bauen fehlgeschlagen: {} Problem(e)", self.probleme.len())); self.schublade = 1; self.einst.schublade = true; false }
            }
        }
    }
    /// Das gebaute Programm im Emulator (oder auf der gewählten Maschine) starten; läuft im Hintergrund
    fn starte(&mut self) -> Task<Msg> {
        let Some(info) = self.info.clone() else { self.log.push("Nichts zum Starten: erst bauen.".into()); return Task::none() };
        let (root, data) = (self.root.clone(), self.data.clone());
        let maschine = vc_ide::settings::effective().start_machine(&self.run_on, &info.target).cloned();
        self.log.push(match &maschine { Some(m) => format!("Starten auf {} …", m.display_name()), None => "Starten im Emulator …".into() });
        Task::perform(async move { tokio::task::spawn_blocking(move || if cfg!(test) { Err("Testlauf: kein Emulator gestartet".to_string()) } else { match maschine {
            Some(m) => vc_ide::launch::run_on_machine(&m, &info.artifact, &info.target),
            None => vc_ide::launch::spawn(&info.artifact, &info.target, &vc_ide::launch::atari_variant(&root), &vc_ide::launch::project_args(&data, &root, &info.target)),
        } }).await.unwrap_or_else(|e| Err(e.to_string())) }, Msg::Gestartet)
    }
    /// System des Projekts (letzter Bau, sonst C64)
    fn system_id(&self) -> String {
        // letzter Bau, sonst das Ziel der Projektdatei (für die offene Datei, sonst das erste), sonst C64
        if let Some(i) = &self.info { return i.target.clone() }
        let Some(pf) = &self.proj.pf else { return "c64".into() };
        let id = self.name().and_then(|n| pf.target_of(&n)).or_else(|| pf.targets.first().map(|t| t.id.clone()));
        id.and_then(|id| pf.targets.iter().find(|t| t.id == id)).map(|t| if t.system.is_empty() { t.id.clone() } else { t.system.clone() }).unwrap_or_else(|| "c64".into())
    }
    /// Anzeigename des Startziels (`""` Standard des Systems, `@emulator`, sonst Maschine)
    fn ziel_name(&self, run_on: &str) -> String {
        let (einst, sys) = (vc_ide::settings::effective(), self.system_id());
        match einst.start_machine(run_on, &sys) {
            Some(m) => m.display_name().to_string(),
            None => { let k = einst.emulator_for(&sys).kind; if k.is_empty() { "Emulator".into() } else { vc_ide::targets::kind_label(&k).to_string() } }
        }
    }
    /// Auswahl der Startziele: Emulator des Systems und die eingerichteten Maschinen
    fn ziel_eintraege(&self) -> Vec<(String, String)> {
        let (einst, sys) = (vc_ide::settings::effective(), self.system_id());
        let mut v = vec![("@emulator".to_string(), format!("Emulator ({})", self.ziel_name("@emulator")))];
        for m in einst.machines_for(&sys) { v.push((m.id.clone(), m.display_name().to_string())) }
        v
    }
    fn ziel_chip(&self, pal: Pal) -> Element<'_, Msg> {
        let sys = self.system_id(); let sname = vc_ide::targets::system(&sys).map(|s| s.name).unwrap_or("C64");
        button(text(format!("{sname} · {} ▾", self.ziel_name(&self.run_on))).size(11.5).color(pal.accent)).padding([3, 10]).on_press(Msg::ZielMenue).style(move |_, _| button::Style { background: None, border: Border { color: pal.accent, width: 1.0, radius: 12.0.into() }, text_color: pal.accent, ..Default::default() }).into()
    }
    /// Haltepunkte der Editoren als Adressen (nur mit Zeilenzuordnung des letzten Baus)
    fn gewuenschte_haltepunkte(&self) -> HashSet<u16> {
        let Some(info) = &self.info else { return HashSet::new() };
        self.editoren.iter().flat_map(|(n, e)| e.breakpoints.iter().filter_map(|l| info.adresse_von(n, l + 1)).collect::<Vec<_>>()).collect()
    }
    /// Debug-Sitzung starten (nach erfolgreichem Bau): Emulator oder Maschine, Haltepunkte, Layout (Schublade und Panel offen)
    fn debug_start(&mut self) {
        let Some(info) = self.info.clone() else { self.log.push("Nichts zum Debuggen: erst bauen.".into()); return };
        if cfg!(test) { self.log.push("Testlauf: keine Debug-Sitzung gestartet".into()); return }
        self.debug_ende();
        vc_ide::launch::stop_running();
        let cpu = match vc_target::load_target(&self.data, &info.target).and_then(|t| vc_target::load_cpu(&self.data, &t.cpu)) { Ok(c) => c, Err(e) => { self.log.push(format!("Debuggen: {e}")); return } };
        let machine = vc_ide::settings::effective().start_machine(&self.run_on, &info.target).cloned();
        if let Some(m) = machine.as_ref().filter(|m| m.kind == "mega65" && m.link_for(vc_ide::mega65net::Action::Debug) == vc_ide::mega65net::Link::Eth) {
            self.log.push(format!("{}: Debuggen geht nur über USB (Ethernet bietet keinen Monitor).", m.display_name())); return
        }
        let (adapter, host, password): (String, Option<String>, Option<String>) = match machine {
            Some(m) if m.kind == "ultimate64" => ("ultimate64".into(), Some(m.host.clone()), Some(m.password.clone())),
            Some(m) if m.kind == "mega65" => ("mega65".into(), vc_ide::settings::mega65_spec(&m).ok(), None),
            _ => (if info.target == "mega65" { "mega65".into() } else { "emulator".into() }, if info.target == "mega65" { Some("xemu".into()) } else { None }, None),
        };
        let spec = vc_ide::debug::LaunchSpec { artifact: info.artifact.to_string_lossy().into_owned(), target: info.target.clone(), adapter, host, password, entry: if info.target == "mega65" { info.run } else { None }, extra_args: vc_ide::launch::project_args(&self.data, &self.root, &info.target) };
        let bps = self.gewuenschte_haltepunkte();
        self.bp_an = bps.clone(); self.reg_alt = None; self.reg_neu = None; self.dbg_fehler = 0; self.dbg_notizen = 0; self.snap = debug::Snap::default();
        self.dbg = Some(debug::Debugger::start(spec, cpu, info.symbols.clone(), bps.into_iter().collect()));
        if !self.beob.is_empty() { if let Some(d) = &self.dbg { d.send(debug::Cmd::Watches(self.beob.clone())) } }
        self.emu_modus_senden();
        self.log.push("Debug-Sitzung startet …".into());
        self.layout.debug_start(); self.schublade = 3;
    }
    fn debug_ende(&mut self) {
        if self.dbg.take().is_none() { return }
        self.test_debug = false;
        self.snap = debug::Snap::default(); self.bp_an.clear(); self.run_to = None;
        self.emu_bild = None; self.reg_alt = None; self.reg_neu = None; self.layout.debug_ende(); if self.schublade == 3 { self.schublade = 0 } if matches!(self.panel_seite, PanelSeite::Debug | PanelSeite::Emulator) { self.panel_seite = PanelSeite::Gliederung } for e in self.editoren.values_mut() { e.debug_line = None }
        self.log.push("Debug-Sitzung beendet".into());
    }
    /// Zustand des Arbeits-Threads übernehmen (alle 100 ms), Haltepunkte abgleichen, Ausführungszeile markieren
    fn debug_abgleich(&mut self) {
        let Some(snap) = self.dbg.as_ref().map(|d| d.snapshot()) else { return };
        if snap.error_seq != self.dbg_fehler { self.dbg_fehler = snap.error_seq; if let Some(e) = &snap.error { self.log.push(e.clone()); if let Some(r) = snap.error_raw.as_ref().filter(|r| *r != e) { self.log.push(format!("  Einzelheit: {r}")) } } }
        else if snap.error_seq == 0 && snap.ended { if let Some(e) = &snap.error { self.log.push(format!("Debuggen: {e}")) } }
        while self.dbg_notizen < snap.notes.len() { self.log.push(snap.notes[self.dbg_notizen].1.clone()); self.dbg_notizen += 1 }
        if snap.ended { self.snap = snap; self.debug_ende(); return }
        // „Bis hierher ausführen“: der einmalige Haltepunkt verschwindet, sobald der Lauf wieder angehalten hat
        if let Some((a, gesehen)) = self.run_to {
            if snap.running { self.run_to = Some((a, true)) }
            else if gesehen && !snap.starting { if let Some(d) = &self.dbg { for p in snap.points.iter().filter(|p| p.kind == "exec" && p.start == a && p.end == a && !self.bp_an.contains(&a)) { d.send(debug::Cmd::DelPoint(p.id)) } } self.run_to = None }
        }
        if !snap.starting {
            let want = self.gewuenschte_haltepunkte();
            if let Some(d) = &self.dbg { for a in want.difference(&self.bp_an) { d.send(debug::Cmd::Bp(*a, true)) } for a in self.bp_an.difference(&want) { d.send(debug::Cmd::Bp(*a, false)) } }
            self.bp_an = want;
        }
        let pcl = if !snap.running && snap.have_regs && !self.test_debug { self.info.as_ref().and_then(|i| i.zeile_bei(snap.pc)).map(|(f, n)| (f.to_string(), n)) } else { None };
        let alt = self.snap.pc; let neu_gehalten = pcl.is_some() && (self.snap.running || alt != snap.pc);
        for (n, e) in self.editoren.iter_mut() { e.debug_line = pcl.as_ref().filter(|(f, _)| f == n).map(|(_, z)| z - 1) }
        // Register: der vorige Haltestand dient dem Vergleich („war …“, geänderte Werte hervorheben)
        if snap.have_regs && !snap.running { let jetzt = [snap.pc, snap.a as u16, snap.x as u16, snap.y as u16, snap.sp as u16, snap.flags as u16]; if self.reg_neu != Some(jetzt) { self.reg_alt = self.reg_neu; self.reg_neu = Some(jetzt) } }
        // Bild des Emulators: nur bei neuer laufender Nummer in eine Bildquelle umwandeln (Messung des Aufbaus)
        if snap.frame_seq != self.emu_bild.as_ref().map_or(0, |b| b.0) {
            if let Some(f) = &snap.frame { let t0 = std::time::Instant::now(); let h = image::Handle::from_rgba(f.width as u32, f.height as u32, f.rgba.clone()); self.emu_ms = if self.emu_ms == 0.0 { t0.elapsed().as_secs_f32() * 1000.0 } else { self.emu_ms * 0.8 + t0.elapsed().as_secs_f32() * 200.0 }; self.emu_bild = Some((snap.frame_seq, h, f.width, f.height)) }
        }
        self.snap = snap;
        if neu_gehalten { if let Some((f, z)) = pcl { self.zeige(&f, z) } }
    }
    /// Ziel-Beschreibung (Bankumschaltung) und Programmbelegung der Speicherkarte aus dem letzten Bau laden
    fn karte_laden(&mut self) {
        let Some(info) = self.info.clone() else { self.karte.facts = None; return };
        if self.karte.target_id != info.target { self.karte.banking = vc_target::load_target(&self.data, &info.target).ok().and_then(|t| t.banking); self.karte.target_id = info.target.clone(); self.karte.preset = None }
        let facts = std::fs::read_to_string(info.artifact.with_extension("vcdbg")).ok().and_then(|t| vc_ide_model::memmap_facts::parse_vcdbg(&t)).unwrap_or_else(|| vc_ide_model::memmap_facts::Facts {
            segs: info.segmente.iter().map(|(a, l)| vc_ide_model::memmap_facts::Seg { start: *a, len: *l, bank: 0, name: String::new() }).collect(),
            syms: info.symbols.iter().filter(|(_, v)| (0..0x10000).contains(*v)).map(|(n, v)| (n.clone(), *v as u32)).collect() });
        if self.karte.facts.is_none() { self.karte.sel = vc_ide_model::memmap_facts::page_usage(&facts.segs).iter().position(|u| *u > 0).unwrap_or(0) as u8 }
        self.karte.facts = Some(facts);
    }
    /// Tests ausführen: Schublade „Tests“ öffnen, ungespeicherte Dateien sichern, Lauf im Arbeits-Thread starten
    fn tests_starten(&mut self, only: vc_ide::tests::suite::Only) -> Task<Msg> {
        for n in self.editoren.keys().cloned().collect::<Vec<_>>() { self.speichere(&n) }
        self.schublade = 2; self.layout.aufklappen(Bereich::Schublade);
        // Tests laufen mit eigenem Emulator (eigener Port, nur eigener Prozess): eine laufende Debug-Sitzung bleibt unberührt
        if self.dbg.is_some() { self.log.push("Die Tests laufen parallel zur Debug-Sitzung (eigener Emulator).".into()) }
        self.tests.start(self.root.clone(), self.data.clone(), only);
        Task::none()
    }
    /// „Diesen Test debuggen“: Emulator mit vorbereiteter Falle, angehalten am Anfang der Falle
    fn test_debug_start(&mut self, file: PathBuf, name: String) {
        // Eine laufende Debug-Sitzung wird nie stillschweigend beendet: erst fragen
        if self.dbg.is_some() { self.rueckfrage = Some(Rueckfrage::TestDebug(file, name)); return }
        self.test_debug_los(file, name)
    }
    fn test_debug_los(&mut self, file: PathBuf, name: String) {
        if cfg!(test) { self.log.push("Testlauf: keine Debug-Sitzung gestartet".into()); return }
        self.tests.stop_blockierend();
        self.bp_an.clear(); self.dbg_fehler = 0; self.dbg_notizen = 0; self.snap = debug::Snap::default(); self.reg_alt = None; self.reg_neu = None; self.test_debug = true;
        self.dbg = Some(debug::Debugger::start_test(file, self.data.clone(), name.clone()));
        self.log.push(format!("Debug-Sitzung für den Test „{name}“ startet …"));
        self.layout.debug_start(); self.schublade = 3;
    }
    fn emu_modus_senden(&self) { if let Some(d) = &self.dbg { d.send(debug::Cmd::FrameMode { auto: self.emu_auto, slow: self.emu_live }) } }
    /// **Genau ein Fokusbesitzer:** der Editor. Danach erhält nur er Tasten-, Text- und IME-Ereignisse.
    fn editor_fokus(&mut self) { self.ed_aktiv = true; self.ed_fokus += 1 }
    /// Ein Textfeld (Suchen, Ersetzen, Gehe zu, Palette, Dialoge, Hilfe, Debug-Felder …) bekommt den Fokus: der Editor verliert ihn.
    fn text_fokus(&mut self, id: &'static str) -> Task<Msg> { self.ed_aktiv = false; iced::widget::operation::focus(id) }
    fn suche_starten(&mut self) -> Task<Msg> {
        if self.suche_frage.trim().is_empty() { return Task::none() }
        self.sucht = true; let (root, eintraege, q) = (self.root.clone(), self.proj.entries.clone(), self.suche_frage.clone());
        let ueber: HashMap<String, String> = self.editoren.iter().filter(|(_, e)| e.dirty).map(|(n, e)| (n.clone(), e.text())).collect();
        Task::perform(async move { tokio::task::spawn_blocking(move || project::suche(&root, &eintraege, ueber, &q, 500)).await.unwrap_or_default() }, Msg::SucheFertig)
    }
    fn palette_eintraege(&self) -> Vec<Eintrag> {
        let mut v: Vec<Eintrag> = befehle::filtere(&self.palette.frage).into_iter().map(|i| Eintrag::Befehl(i.cmd)).collect();
        let q = self.palette.frage.to_lowercase();
        v.extend(self.studio_aktionen().into_iter().filter(|x| x.aktiv && (q.is_empty() || format!("studio {}", x.name).to_lowercase().contains(&q) || x.name.to_lowercase().contains(&q))).map(|x| Eintrag::Studio(x.name, befehle::plattform_text(&x.kuerzel), Box::new(x.msg))));
        if !q.is_empty() { v.extend(self.namen.iter().filter(|n| n.to_lowercase().contains(&q)).take(12).map(|n| Eintrag::Datei(n.clone()))) }
        if !cfg!(test) && (q.len() >= 2 || q == "z") { for r in vc_ide_model::recent::load(Some(&self.data)).into_iter().filter(|r| !r.missing && (r.name.to_lowercase().contains(&q) || r.path.to_string_lossy().to_lowercase().contains(&q) || "zuletzt geöffnet".contains(&q))).take(6) { v.push(Eintrag::Projekt(r.path, r.name)) } }
        v
    }
    fn gliederung_holen(&self) -> Vec<(usize, String)> {
        let Some(Tab::Datei(n)) = self.tabs.get(self.aktiv) else { return vec![] }; let Some(e) = self.editoren.get(n) else { return vec![] };
        let mut c = self.gliederung.borrow_mut(); if c.0 != *n || c.1 != e.version { c.2 = e.lines.iter().enumerate().filter_map(|(i, l)| { let s = l.strip_suffix(':').or_else(|| l.split(':').next().filter(|_| l.contains(':'))); s.filter(|s| !s.is_empty() && !s.starts_with(char::is_whitespace) && !s.starts_with(';') && s.chars().all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.')).map(|s| (i, s.to_string())) }).collect(); c.0 = n.clone(); c.1 = e.version }
        c.2.clone()
    }

    // ------------------------------------------------------------ Kopfzeile
    /// Breitenstufe der Kopfzeile: 0 voll, 1 Symbole mit Tooltip, 2 zusätzlich Suchfeld als Lupe, 3 zusätzlich Debuggen/Bauen im „⋯“-Menü
    fn kopf_stufe(&self) -> u8 { let w = self.fenster.width / self.einst.zoom; if w >= 1180.0 { 0 } else if w >= 960.0 { 1 } else if w >= 640.0 { 2 } else { 3 } }
    fn kopf_eintraege(&self) -> Vec<(Cmd, &'static str, bool)> {
        vec![(Cmd::Debuggen, "● Debuggen", true), (Cmd::Bauen, if self.baut { t::BAUT } else { t::BAUEN }, !self.baut)]
    }
    fn kopfzeile<'a>(&'a self, pal: Pal) -> Element<'a, Msg> {
        let stufe = self.kopf_stufe();
        fn tipp_fn<'a>(pal: Pal, inhalt: Element<'a, Msg>, name: &str, cmd: Cmd) -> Element<'a, Msg> {
            let tasten = befehle::BEFEHLE.iter().find(|i| i.cmd == cmd).map(|i| befehle::fuer_anzeige(i.tasten, cfg!(target_os = "macos"))).unwrap_or_default();
            iced::widget::tooltip(inhalt, container(text(if tasten.is_empty() { name.to_string() } else { format!("{name}   {tasten}") }).size(12).color(pal.text)).padding([4, 8]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Bottom).into()
        }
        let tipp = |inhalt: Element<'a, Msg>, name: &str, cmd: Cmd| -> Element<'a, Msg> { tipp_fn(pal, inhalt, name, cmd) };
        let ziffer = |s: &'static str| -> &'static str { s.split_once(' ').map(|(a, _)| a).unwrap_or(s) };
        let name = if self.root == ohne_projekt_ordner() { "Kein Projekt".to_string() } else { self.root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() };
        let kuerzen = |n: &str, max: usize| -> String { if n.chars().count() > max { format!("{}…", n.chars().take(max.saturating_sub(1)).collect::<String>()) } else { n.to_string() } };
        let (max_name, chip_kurz) = match stufe { 0 => (40, false), 1 => (if self.dbg.is_some() { 14 } else { 22 }, self.dbg.is_some()), 2 => (14, true), _ => (10, true) };
        let projekt = column![text(kuerzen(&name, max_name)).size(14).color(pal.text).wrapping(iced::widget::text::Wrapping::None), text("Projekt ▾").size(11).color(pal.mute)].spacing(1);
        let aktionen: Element<Msg> = if self.dbg.is_some() {
            if stufe >= 1 { self.debug_leiste_kompakt(pal) } else { self.debug_leiste(pal) }
        } else {
            let start = if stufe == 0 { knopf(pal, t::STARTEN, Some(Msg::Befehl(Cmd::Starten)), pal.text, true) } else { tipp(knopf(pal, ziffer(t::STARTEN), Some(Msg::Befehl(Cmd::Starten)), pal.text, true), "Starten", Cmd::Starten) };
            let mut r = row![start].spacing(if stufe == 0 { 10 } else { 6 }).align_y(Alignment::Center);
            if stufe < 3 {
                r = r.push(if stufe == 0 { button(row![dbgview::ikon(dbgview::Ikon::Kaefer, pal.text), text("Debuggen").size(13).color(pal.text)].spacing(6).align_y(Alignment::Center)).padding([6, 12]).on_press(Msg::Befehl(Cmd::Debuggen)).style(move |_, _| button::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() } else { tipp(button(container(dbgview::ikon(dbgview::Ikon::Kaefer, pal.text)).center_x(Length::Shrink)).padding([7, 12]).on_press(Msg::Befehl(Cmd::Debuggen)).style(move |_, _| button::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into(), "Debuggen", Cmd::Debuggen) });
                let (bt, an) = (if self.baut { t::BAUT } else { t::BAUEN }, (!self.baut).then_some(Msg::Befehl(Cmd::Bauen)));
                r = r.push(if stufe == 0 { knopf(pal, bt, an, pal.text, false) } else { tipp(knopf(pal, ziffer(bt), an, pal.text, false), "Bauen", Cmd::Bauen) });
            } else { r = r.push(tipp(knopf(pal, "⋯", Some(Msg::KopfMenue), pal.text, false), "Mehr", Cmd::Palette)) }
            r.into()
        };
        let suche: Element<Msg> = if stufe >= 2 || self.dbg.is_some() && stufe >= 1 {
            tipp(knopf(pal, "⌕", Some(Msg::Befehl(Cmd::Palette)), pal.text, false), t::SUCHFELD, Cmd::Palette)
        } else if stufe == 1 {
            button(text(format!("{}   {}", "Suchen", befehle::fuer_anzeige("Cmd+K", cfg!(target_os = "macos") && !self.kuerzel_windows))).size(12).color(pal.mute)).on_press(Msg::Befehl(Cmd::Palette)).style(move |_, _| button::Style { background: Some(pal.editor.into()), border: Border { color: pal.line, width: 1.0, radius: 15.0.into() }, text_color: pal.mute, ..Default::default() }).into()
        } else {
            button(text(format!("{}        {}", t::SUCHFELD, befehle::fuer_anzeige("Cmd+K", cfg!(target_os = "macos") && !self.kuerzel_windows))).size(12).color(pal.mute)).on_press(Msg::Befehl(Cmd::Palette)).style(move |_, _| button::Style { background: Some(pal.editor.into()), border: Border { color: pal.line, width: 1.0, radius: 15.0.into() }, text_color: pal.mute, ..Default::default() }).into()
        };
        let chip: Element<Msg> = if chip_kurz { tipp(self.ziel_chip_kurz(pal), &format!("Ziel: {} · {}", vc_ide::targets::system(&self.system_id()).map(|s| s.name).unwrap_or("C64"), self.ziel_name(&self.run_on)), Cmd::Palette) } else { self.ziel_chip(pal) };
        container(row![
            image(self.logos[if stufe >= 2 { 2 } else if pal.dark { 0 } else { 1 }].clone()).height(24),
            projekt, chip, aktionen, space::horizontal(), suche, knopf(pal, "⚙", Some(Msg::Befehl(Cmd::Einstellungen)), pal.text, false),
        ].spacing(if stufe == 0 { 10 } else { 8 }).align_y(Alignment::Center)).padding([8, if stufe >= 2 { 10 } else { 14 }]).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into()
    }
    fn ziel_chip_kurz(&self, pal: Pal) -> Element<'_, Msg> {
        let sys = self.system_id(); let sname = vc_ide::targets::system(&sys).map(|s| s.name).unwrap_or("C64");
        let kurz = match sys.as_str() { "c64" => "C64", "c128" => "C128", "vic20" => "VC-20", "plus4" => "Plus/4", "c16" => "C16", "atari8" => "Atari", "mega65" => "MEGA65", "pet" => "PET", _ => sname };
        button(text(format!("{kurz} ▾")).size(11.5).color(pal.accent)).padding([3, 8]).on_press(Msg::ZielMenue).style(move |_, _| button::Style { background: None, border: Border { color: pal.accent, width: 1.0, radius: 12.0.into() }, text_color: pal.accent, ..Default::default() }).into()
    }

    // ------------------------------------------------------------ Ansicht
    fn view(&self) -> Element<'_, Msg> {
        vc_gui_ui::muster::zen_setzen(self.layout.zen());
        let pal = self.pal(); let sz = self.einst.editor_schrift;
        let kopf = self.kopfzeile(pal);

        let zen = self.layout.zen(); let anz = |b: Bereich| self.layout.anzeige(b);
        let (sb, sl, pa, doc) = (anz(Bereich::Seitenleiste), anz(Bereich::Schublade), anz(Bereich::Panel), anz(Bereich::Dokument));
        // Unter 800 px Breite bleibt von der Seitenleiste nur der Streifen (sonst bekommt die Arbeitsfläche keinen Platz); Maximieren zeigt sie weiter
        let sb = if sb == Anzeige::Normal && self.fenster.width / self.einst.zoom < 800.0 { Anzeige::Minimiert } else { sb };
        let mitte_sichtbar = sb != Anzeige::Maximiert && pa != Anzeige::Maximiert;
        let leiste_nur = doc == Anzeige::Minimiert;
        let mut mitte = column![];
        if !leiste_nur && self.find.offen { mitte = mitte.push(self.find_leiste(pal)) }
        if !leiste_nur && self.goto.offen { mitte = mitte.push(self.goto_leiste(pal)) }
        let gruppen: Element<Msg> = match &self.andere {
            None => self.gruppe_ansicht(pal, sz, false, &self.tabs, self.aktiv, true, leiste_nur),
            Some((at, aa)) => {
                let (l, r) = if self.fokus_rechts { (self.gruppe_ansicht(pal, sz, false, at, *aa, false, leiste_nur), self.gruppe_ansicht(pal, sz, true, &self.tabs, self.aktiv, true, leiste_nur)) } else { (self.gruppe_ansicht(pal, sz, false, &self.tabs, self.aktiv, true, leiste_nur), self.gruppe_ansicht(pal, sz, true, at, *aa, false, leiste_nur)) };
                row![l, container(space::horizontal()).width(1).height(Length::Fill).style(move |_| flaeche(pal.line, pal.line)), r].into()
            }
        };
        let gruppen: Element<Msg> = if !leiste_nur && self.ziehen.as_ref().is_some_and(|z| z.start.is_some()) && self.andere.is_none() {
            stack![gruppen, container(container(text(t::ABLEGEN_RECHTS).size(13).color(pal.accent)).center(Length::Fill).width(Length::FillPortion(35)).height(Length::Fill).style(move |_| flaeche(Color { a: 0.12, ..pal.accent }, pal.accent))).align_right(Length::Fill).padding([36, 0])].into()
        } else { gruppen };
        // Ist die Mitte ausgeblendet (Spalte oder Panel maximiert), stehen Reiterleiste und Schubladenleiste oben und unten
        let mut oben: Option<Element<Msg>> = None; let mut unten: Option<Element<Msg>> = None;
        if mitte_sichtbar { mitte = mitte.push(gruppen); if sl == Anzeige::Normal { if self.schublade == 3 && self.dbg.is_some() { mitte = mitte.push(iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(5).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() })).on_press(Msg::TrennerStart).interaction(iced::mouse::Interaction::ResizingVertically)) } mitte = mitte.push(self.schublade_ansicht(pal, false)) } else if sl == Anzeige::Maximiert { mitte = mitte.push(self.schublade_ansicht(pal, true)) } else if !zen { mitte = mitte.push(self.schublade_leiste(pal)) } }
        else { oben = Some(gruppen); unten = Some(self.schublade_leiste(pal)) }
        // bei maximierter Schublade bekommt sie den Platz: das Dokumentenfeld ist nur noch Leiste (siehe Layoutregeln)
        let mut koerper = row![];
        // Zen-Modus: auch die Streifen von Spalte, Schublade und Panel verschwinden; nur Reiter und Fläche bleiben
        if !zen { koerper = koerper.push(match sb { Anzeige::Normal => self.seitenleiste(pal, false), Anzeige::Minimiert => self.spalten_streifen(pal), Anzeige::Maximiert => self.seitenleiste(pal, true) }); }
        if mitte_sichtbar {
            let mitte_el: Element<Msg> = if pa == Anzeige::Normal && self.schmal() { stack![mitte.width(Length::Fill).height(Length::Fill), container(self.panel(pal, false)).align_right(Length::Fill)].into() } else { mitte.width(Length::Fill).into() };
            koerper = koerper.push(mitte_el);
        }
        match pa { Anzeige::Normal if !self.schmal() => koerper = koerper.push(self.panel(pal, false)), Anzeige::Maximiert => koerper = koerper.push(self.panel(pal, true)), _ => {} }
        if !zen { koerper = koerper.push(self.streifen(pal)); }
        let koerper: Element<Msg> = if oben.is_some() || unten.is_some() { let mut c = column![]; if let Some(o) = oben { c = c.push(o) } c = c.push(koerper.height(Length::Fill)); if let Some(u) = unten { c = c.push(u) } c.into() } else { koerper.into() };

        let (z, s) = self.name().and_then(|n| self.editoren.get(&n)).map(|e| (e.cursor.line + 1, e.cursor.col + 1)).unwrap_or((0, 0));
        let upd_hinweis: Element<Msg> = match self.update_hinweis() { Some(v) => button(text(format!("Update {v} verfügbar")).size(11.5).color(pal.accent)).padding([0, 8]).on_press(Msg::Info(infodlg::InfoAkt::UpdateAuf)).style(|_, _| button::Style::default()).into(), None => text("").into() };
        let status = container(row![text(if !self.hinweis.is_empty() { self.hinweis.clone() } else if self.baut { "baut …".to_string() } else { t::BEREIT.to_string() }).size(11.5).color(pal.mute), upd_hinweis, space::horizontal(), self.takt_anzeige(pal), space::horizontal().width(12), { let mt = self.name().and_then(|n| self.textmeta.get(&n).copied()).unwrap_or_default(); let schalter = |t: &str, tipp: &str, c: Cmd| -> Element<Msg> { iced::widget::tooltip(button(text(t.to_string()).size(11.5).color(pal.mute)).padding([0, 6]).on_press(Msg::Befehl(c)).style(|_, _| button::Style::default()), container(text(tipp.to_string()).size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Top).into() }; let r: Element<Msg> = if z > 0 { row![text(format!("Z {z} · S {s}   {}", self.editoren.get(&self.name().unwrap_or_default()).map_or("", |e| match e.lang { vc_ide_model::lang::Lang::Asm => "Assembler", _ => "Text" }))).size(11.5).color(pal.mute), schalter(mt.kodierung(), "Kodierung beim Speichern: UTF-8 ↔ ISO-8859-1 (Klick wechselt)", Cmd::Kodierung), schalter(mt.zeilenende(), "Zeilenende beim Speichern: LF ↔ CRLF (Klick wechselt)", Cmd::Zeilenende)].spacing(6).align_y(Alignment::Center).into() } else { text("").into() }; r }]).padding([4, 12]).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line));
        let grund: Element<Msg> = container({ let mut c = column![]; if self.menue_sichtbar() { c = c.push(self.menueleiste(pal)) } c.push(kopf).push(container(koerper).height(Length::Fill)).push(status) }).style(move |_| container::Style { background: Some(pal.bg.into()), ..Default::default() }).into();
        let grund: Element<Msg> = if self.ziehen.is_some() { stack![grund, iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill)).on_move(Msg::Maus).on_release(Msg::MausLos).interaction(iced::mouse::Interaction::Grabbing)].into() } else { grund };
        let grund: Element<Msg> = if let Some(k) = &self.kontext {
            let (zeilen, _) = self.kontext_zeilen(k); let hoehe = zeilen.iter().map(|z| if z.name.is_empty() { 3.0 } else { 29.0 }).sum::<f32>() + 10.0;
            let breite = if k.art == Art::Menue { 430.0 } else { 260.0 }; let (x, y) = (k.pos.x.min(self.fenster.width / self.einst.zoom - breite - 8.0).max(4.0), if k.art == Art::Menue { k.pos.y } else { k.pos.y.min(self.fenster.height / self.einst.zoom - hoehe - 8.0).max(4.0) });
            let fang = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill)).on_press(Msg::KontextZu).on_right_press(Msg::KontextZu);
            let karte: Element<Msg> = if k.art == Art::Menue && hoehe > self.fenster.height / self.einst.zoom - y - 8.0 { scrollable(kontext::menue_breit(pal, zeilen, k.sel, breite)).id("menue").height(Length::Fixed((self.fenster.height / self.einst.zoom - y - 8.0).max(120.0))).into() } else { kontext::menue_breit(pal, zeilen, k.sel, breite) };
            stack![grund, fang, container(karte).padding(iced::Padding { left: x, top: y, ..Default::default() })].into()
        } else { grund };
        let grund: Element<Msg> = if self.ziel_menue {
            let zeilen: Vec<Element<Msg>> = self.ziel_eintraege().into_iter().map(|(id, name)| { let sel = id == self.run_on || (self.run_on.is_empty() && id == "@emulator"); button(text(format!("{}{name}", if sel { "✓ " } else { "   " })).size(13).color(pal.text)).width(Length::Fill).padding([6, 12]).on_press(Msg::ZielWahl(id)).style(move |_, st| button::Style { background: matches!(st, button::Status::Hovered).then(|| pal.sel.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() }).collect();
            let fang = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill)).on_press(Msg::ZielMenue);
            stack![grund, fang, container(container(column(zeilen).spacing(2).padding(6)).width(280).style(move |_| flaeche(pal.panel, pal.line))).padding(iced::Padding { left: 190.0, top: 46.0, ..Default::default() })].into()
        } else { grund };
        let grund: Element<Msg> = if self.kopf_menue {
            let zeilen: Vec<Element<Msg>> = self.kopf_eintraege().into_iter().map(|(cmd, name, an)| {
                let tasten = befehle::BEFEHLE.iter().find(|i| i.cmd == cmd).map(|i| befehle::fuer_anzeige(i.tasten, cfg!(target_os = "macos") && !self.kuerzel_windows)).unwrap_or_default();
                let mut b = button(row![text(name).size(13).color(pal.text), space::horizontal(), text(tasten).size(12).color(pal.mute)].spacing(16)).width(Length::Fill).padding([6, 12]).style(move |_, st| button::Style { background: matches!(st, button::Status::Hovered).then(|| pal.sel.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() });
                if an { b = b.on_press(Msg::KopfWahl(cmd)) } b.into() }).collect();
            let fang = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill)).on_press(Msg::KopfMenue);
            stack![grund, fang, container(container(column(zeilen).spacing(2).padding(6)).width(240).style(move |_| flaeche(pal.panel, pal.line))).padding(iced::Padding { left: 150.0, top: 46.0, ..Default::default() })].into()
        } else { grund };
        let grund: Element<Msg> = match self.baum_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.import_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.export_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.song_import_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.rueckfrage_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.scratch_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.neuprojekt_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.wiz_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.info_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.kopier_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.zeichen_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.hexv_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.beenden_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = match self.zahlen_dialog(pal) { Some(d) => stack![grund, d].into(), None => grund };
        let grund: Element<Msg> = if self.splash {
            let karte = container(column![image(self.logos[2].clone()).height(96), text("VintageCoder").size(30).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), text(format!("Version {}", infodlg::version())).size(13).color(pal.mute), text("Alpha-Version").size(12).color(pal.warn), text("Klick oder Taste schließt das Startbild").size(11).color(pal.mute)].spacing(8).align_x(Alignment::Center)).padding(36).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 16.0.into() }, shadow: iced::Shadow { color: Color { a: 0.45, ..Color::BLACK }, offset: iced::Vector::new(0.0, 8.0), blur_radius: 24.0 }, ..Default::default() });
            stack![grund, iced::widget::mouse_area(container(container(karte).center(Length::Fill)).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.55, ..Color::BLACK }.into()), ..Default::default() })).on_press(Msg::SplashZu)].into()
        } else { grund };
        if self.palette.offen { stack![grund, self.palette_ansicht(pal)].into() } else { grund }
    }

    /// Reiterleiste und Inhalt einer Gruppe; `rechts` sagt, welche Seite, `fokus` ob sie die Tastatur hat
    fn gruppe_ansicht<'a>(&'a self, pal: Pal, sz: f32, rechts: bool, tabs: &'a [Tab], aktiv: usize, fokus: bool, nur_leiste: bool) -> Element<'a, Msg> {
        let (lo, hi) = self.sichtbare_reiter(tabs, aktiv);
        let leiste = row(tabs.iter().enumerate().skip(lo).take(hi - lo).map(|(i, x)| {
            let (name, dirty) = match x { Tab::Datei(n) => (n.rsplit('/').next().unwrap_or(n).to_string(), (self.editoren.get(n).map_or(false, |e| e.dirty) || self.studios.get(n).map_or(false, |s| s.doc.dirty()) || self.paletten.get(n).map_or(false, |p| p.dirty()) || self.disks.get(n).map_or(false, |d| d.dirty))), Tab::Einstellungen => (t::EINSTELLUNGEN.to_string(), false), Tab::Start => ("Start".to_string(), false), Tab::Projekt => ("Projekt".to_string(), self.pe.as_ref().is_some_and(|z| z.geaendert())), Tab::Hilfe(k) => (self.hilfe_titel(k), false) };
            let sel = i == aktiv; let gezogen = self.ziehen.as_ref().is_some_and(|z| z.start.is_some() && z.tab == *x);
            let titel = container(text(format!("{name}{}", if dirty { " •" } else { "" })).size(12.5).color(if sel { pal.text } else { pal.mute })).padding([6, 10]).style(move |_| container::Style { background: (sel || gezogen).then(|| if gezogen { pal.sel.into() } else { pal.editor.into() }), border: Border { radius: 6.0.into(), color: if sel && fokus { pal.accent } else { Color::TRANSPARENT }, width: if sel && fokus { 1.0 } else { 0.0 } }, ..Default::default() });
            row![kontext::rechtsklick(iced::widget::mouse_area(titel).on_press(Msg::TabDruck(rechts, i)).on_enter(Msg::TabUeber(Some((rechts, i)))).on_exit(Msg::TabUeber(None)).interaction(iced::mouse::Interaction::Pointer), move |pt| Msg::KontextReiter(rechts, i, pt)),
                 button(text("×").size(13).color(pal.mute)).padding([4, 6]).on_press(Msg::SchliesseIn(rechts, i)).style(move |_, _| button::Style { background: None, text_color: pal.mute, ..Default::default() })].align_y(Alignment::Center).into() }).collect::<Vec<_>>()).spacing(4);
        let ueberlauf = |n: usize, links: bool, ziel: usize| -> Element<'a, Msg> { button(text(if links { format!("‹ {n}") } else { format!("{n} ›") }).size(12).color(pal.accent)).padding([6, 8]).on_press(Msg::ReiterIn(rechts, ziel)).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.accent, ..Default::default() }).into() };
        let teilen: Element<'a, Msg> = iced::widget::tooltip(button(text("◫").size(14).color(pal.mute)).padding([4, 8]).on_press(Msg::Befehl(Cmd::Teilen)).style(move |_, _| button::Style { background: None, text_color: pal.mute, ..Default::default() }), container(text(if self.andere.is_some() { "Teilung aufheben (Cmd+Alt+T)" } else { "Editor teilen: zweite Hälfte rechts (Cmd+Alt+T)" }).size(12).color(pal.text)).padding([5, 8]).style(move |_| container::Style { background: Some(pal.editor.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, ..Default::default() }), iced::widget::tooltip::Position::Bottom).into();
        let mut zeile = row![].spacing(4).align_y(Alignment::Center);
        if lo > 0 { zeile = zeile.push(ueberlauf(lo, true, lo - 1)) }
        zeile = zeile.push(leiste);
        if hi < tabs.len() { zeile = zeile.push(ueberlauf(tabs.len() - hi, false, hi)) }
        zeile = zeile.push(teilen);
        let leiste = zeile;
        let leiste = bw::kopfzeile(pal, t::DOKUMENTE, self.layout.anzeige(Bereich::Dokument), leiste.into(), false, "", KUERZEL_MAX, akt(Bereich::Dokument));
        let leiste = container(leiste).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line));
        if nur_leiste { return container(leiste).width(Length::FillPortion(1)).into() }
        let inhalt: Element<Msg> = match tabs.get(aktiv) {
            Some(Tab::Datei(n)) if self.studios.contains_key(n) => iced::widget::mouse_area(self.studio_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.paletten.contains_key(n) => iced::widget::mouse_area(self.farbpalette_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.songs.contains_key(n) => iced::widget::mouse_area(self.song_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.banken.contains_key(n) => iced::widget::mouse_area(self.bank_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.tilesets.contains_key(n) => iced::widget::mouse_area(self.tile_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.screens.contains_key(n) => iced::widget::mouse_area(self.screen_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.hexe.contains_key(n) => iced::widget::mouse_area(self.hex_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.disks.contains_key(n) => iced::widget::mouse_area(self.disk_ansicht(pal, n)).on_press(Msg::GruppeKlick(rechts)).into(),
            Some(Tab::Datei(n)) if self.md_vorschau(n) => { let ed = &self.editoren[n]; let doc = self.md_doc(&ed.text());
                let leiste = container(row![text("Markdown-Vorschau").size(12).color(pal.mute), space::horizontal(), vc_gui_ui::muster::knopf(pal, "Quelltext", "Zum Editor wechseln (Befehlspalette: Markdown: Vorschau/Quelltext)", Some(Msg::MdUmschalten(n.clone())), false, false)].align_y(Alignment::Center)).padding([4, 12]).width(Length::Fill).style(move |_| flaeche(pal.panel2, pal.line));
                column![leiste, container(scrollable(container(mdansicht::ansicht(pal, &doc, self.md_grenze, Some(Msg::MdMehr))).padding(iced::Padding { top: 12.0, right: 24.0, bottom: 24.0, left: 24.0 }).max_width(820)).height(Length::Fill)).width(Length::Fill).height(Length::Fill).style(move |_| flaeche(pal.editor, pal.line))].into() }
            Some(Tab::Datei(n)) if n.to_lowercase().ends_with(".md") && self.editoren.contains_key(n) => { let ed = &self.editoren[n]; let v = editor_view(ed, pal, sz, move |m| Msg::EdIn(rechts, m)).aktiv(self.ed_aktiv).ziehen(self.baum_zug.as_ref().is_some_and(|g| !g.ordner)).alt_menue(self.alt_menue_buchstaben()); let ev = container(if fokus { v.fokus(self.ed_fokus) } else { v }).width(Length::Fill).height(Length::Fill);
                let leiste = container(row![text("Markdown-Quelltext").size(12).color(pal.mute), space::horizontal(), vc_gui_ui::muster::knopf(pal, "Vorschau", "Zur gerenderten Ansicht wechseln", Some(Msg::MdUmschalten(n.clone())), false, false)].align_y(Alignment::Center)).padding([4, 12]).width(Length::Fill).style(move |_| flaeche(pal.panel2, pal.line));
                column![leiste, iced::widget::tooltip(ev, self.hover_karte(pal, ed), iced::widget::tooltip::Position::FollowCursor).gap(18).snap_within_viewport(true)].into() }
            Some(Tab::Datei(n)) => match self.editoren.get(n) { Some(ed) => { let v = editor_view(ed, pal, sz, move |m| Msg::EdIn(rechts, m)).aktiv(self.ed_aktiv).ziehen(self.baum_zug.as_ref().is_some_and(|g| !g.ordner)).alt_menue(self.alt_menue_buchstaben()); let ev = container(if fokus { v.fokus(self.ed_fokus) } else { v }).width(Length::Fill).height(Length::Fill); iced::widget::tooltip(ev, self.hover_karte(pal, ed), iced::widget::tooltip::Position::FollowCursor).gap(18).snap_within_viewport(true).into() } None => text("").into() },
            Some(Tab::Einstellungen) => self.einstellungen_ansicht(pal),
            Some(Tab::Start) => self.startseite(pal),
            Some(Tab::Projekt) => self.pe_ansicht(pal),
            Some(Tab::Hilfe(k)) => self.hilfe_reiter(pal, k),
            None => self.startseite(pal),
        };
        column![leiste, inhalt].width(Length::FillPortion(1)).height(Length::Fill).into()
    }
    fn find_leiste(&self, pal: Pal) -> Element<'_, Msg> {
        let mut z = column![row![
            text_input(t::FINDEN, &self.find.frage).id("find").on_input(Msg::FindFrage).on_submit(Msg::FindSchritt(1)).width(260).padding(6),
            text(self.find.info.clone()).size(12).color(pal.mute).width(90),
            knopf(pal, "↑", Some(Msg::FindSchritt(-1)), pal.text, false), knopf(pal, "↓", Some(Msg::FindSchritt(1)), pal.text, false),
            knopf(pal, if self.find.gross_klein { "Aa ✓" } else { "Aa" }, Some(Msg::FindGrossKlein), pal.text, false),
            space::horizontal().width(8), knopf(pal, "×", Some(Msg::FindZu), pal.mute, false),
        ].spacing(8).align_y(Alignment::Center).wrap().vertical_spacing(6)].spacing(6);
        if self.find.ersetzen { z = z.push(row![text_input(t::ERSETZEN, &self.find.mit).on_input(Msg::ErsetzFrage).on_submit(Msg::ErsetzeEins).width(260).padding(6), knopf(pal, t::ERSETZEN, Some(Msg::ErsetzeEins), pal.text, false), knopf(pal, t::ALLE, Some(Msg::ErsetzeAlle), pal.text, false)].spacing(8).align_y(Alignment::Center)) }
        container(z).padding([8, 12]).width(Length::Fill).style(move |_| flaeche(pal.panel2, pal.line)).into()
    }
    fn goto_leiste(&self, pal: Pal) -> Element<'_, Msg> {
        container(row![text("Gehe zu Zeile").size(12.5).color(pal.mute), text_input("Zeilennummer", &self.goto.text).id("goto").on_input(Msg::GotoText).on_submit(Msg::GotoLos).width(140).padding(6)].spacing(10).align_y(Alignment::Center)).padding([8, 12]).width(Length::Fill).style(move |_| flaeche(pal.panel2, pal.line)).into()
    }
    fn schublade_ansicht(&self, pal: Pal, voll: bool) -> Element<'_, Msg> {
        let body: Element<Msg> = if self.schublade == 3 && self.dbg.is_some() { self.debug_flaeche(pal) } else if self.schublade == 2 { self.tests_ansicht(pal) } else if self.schublade == 0 {
            kontext::rechtsklick(scrollable(column(self.log.iter().rev().take(300).rev().map(|l| text(l.clone()).size(12).font(vc_gui_ui::fonts::MONO).color(pal.text).into()).collect::<Vec<_>>()).spacing(2).padding(8)).anchor_bottom().width(Length::Fill), |pt| Msg::KontextAuf(Art::Protokoll, pt, None))
        } else if self.probleme.is_empty() { container(text("Keine Probleme.").size(12.5).color(pal.mute)).padding(12).into() } else {
            scrollable(column(self.probleme.iter().enumerate().map(|(i, p)| kontext::rechtsklick(button(text(format!("{}:{}  {}", p.datei, p.zeile, p.text)).size(12).font(vc_gui_ui::fonts::MONO).color(if p.fehler { pal.err } else { pal.warn })).on_press(Msg::GotoProblem(i)).style(move |_, _| button::Style { background: None, text_color: pal.text, ..Default::default() }), move |pt| Msg::KontextAuf(Art::Problem, pt, Some(i.to_string())))).collect::<Vec<_>>()).spacing(2).padding(8)).width(Length::Fill).into()
        };
        let kopf = self.schublade_kopf(pal);
        container(column![kopf, body]).height(if voll { Length::Fill } else { Length::Fixed(if self.schublade == 3 && self.dbg.is_some() { (self.einst.debug_hoehe * ((self.fenster.height / self.einst.zoom) - 78.0)).max(180.0) } else if self.schublade == 2 { 280.0 } else { 170.0 }) }).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into()
    }
    /// Test-Explorer: Leiste (Lauf, Emulator, Filter), Liste nach Gruppen mit Zuständen, Detail mit Erwartung, Registern und Zyklen
    fn tests_ansicht(&self, pal: Pal) -> Element<'_, Msg> {
        use vc_ide::tests::{Level, State};
        let tl = &self.tests; let mono = vc_gui_ui::fonts::MONO;
        let knopf_k = |name: String, m: Option<Msg>| -> Element<Msg> { let an = m.is_some(); let mut b = button(text(name).size(11.5).color(if an { pal.text } else { pal.mute })).padding([3, 8]).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }); if let Some(m) = m { b = b.on_press(m) } b.into() };
        let farbe = |o: &vc_ide::tests::Outcome| match (o.state, o.level) { (State::Passed, _) => pal.ok, (State::Failed, Level::Error) => pal.err, (State::Failed, Level::Hint) | (State::TimedOut, _) => pal.warn, (State::Skipped, _) => pal.mute };
        let c = tl.suite.counts(); let fehler = c.failed + c.timed_out;
        let leiste = row![
            if tl.laeuft() { knopf_k("■ Abbrechen".into(), Some(Msg::TestStopp)) } else if self.dbg.is_some() { iced::widget::tooltip(knopf_k("▶ Alle ausführen".into(), Some(Msg::TestAlle)), container(text(t::TESTS_PARALLEL).size(11.5).color(pal.text)).padding(6).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Top).into() } else { knopf_k("▶ Alle ausführen".into(), Some(Msg::TestAlle)) },
            knopf_k("Fehlgeschlagene".into(), (!tl.laeuft() && fehler > 0).then_some(Msg::TestFehler)),
            knopf_k(format!("Läuft auf: {} ▾", tl.backend_name()), Some(Msg::TestBackend)),
            text_input("Filter", &tl.filter).on_input(Msg::TestFilter).size(12).padding(3).width(150),
            iced::widget::checkbox(tl.nur_fehler).label("nur Fehler").on_toggle(Msg::TestNurFehler).size(14).text_size(11.5),
            space::horizontal(),
            text(if tl.laeuft() { "läuft …".to_string() } else { format!("{} bestanden · {} fehlgeschlagen · {} übersprungen{}", c.passed, fehler, c.skipped, tl.suite.last_millis.map(|m| format!(" · {}", testlauf::dauer(m))).unwrap_or_default()) }).size(11.5).color(pal.mute),
        ].spacing(8).align_y(Alignment::Center);
        // Liste
        let mut liste = column![].spacing(1); let mut letzte: Option<usize> = None;
        if let Some(e) = &tl.suite.error { liste = liste.push(text(e.clone()).size(12).color(pal.err)) }
        if tl.suite.is_empty() && !tl.laeuft() && tl.suite.error.is_none() { liste = liste.push(text(t::TESTS_LEER).size(12.5).color(pal.mute)) }
        for (gi, ii) in tl.suite.visible(&tl.filter, tl.nur_fehler) {
            let g = &tl.suite.groups[gi];
            if letzte != Some(gi) { letzte = Some(gi); liste = liste.push(text(format!("{}{}{}", g.title, g.backend.as_ref().map(|b| format!(" · {b}")).unwrap_or_default(), if g.expected > g.items.len() { format!(" ({}/{})", g.items.len(), g.expected) } else { String::new() })).size(11).color(pal.mute)) }
            let it = &g.items[ii]; let o = &it.outcome; let gewaehlt = tl.sel.as_ref().is_some_and(|(f, n)| *f == it.file && *n == o.name);
            let zeile = row![text(testlauf::zustand_zeichen(o)).size(12).color(farbe(o)), text(o.name.clone()).size(12.5).color(pal.text), space::horizontal(), text(o.measured.clone().unwrap_or_default()).size(11.5).font(mono).color(pal.mute)].spacing(8);
            liste = liste.push(button(zeile).width(Length::Fill).padding([2, 6]).on_press(Msg::TestWahl(it.file.clone(), o.name.clone())).style(move |_, st| button::Style { background: if gewaehlt { Some(pal.sel.into()) } else { matches!(st, button::Status::Hovered).then(|| pal.panel2.into()) }, border: Border { radius: 4.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }));
        }
        // Detail
        let mut detail = column![].spacing(5);
        let gew = tl.sel.as_ref().and_then(|(f, n)| tl.suite.groups.iter().find_map(|g| g.items.iter().find(|i| i.file == *f && i.outcome.name == *n).map(|i| (g, i))));
        match gew {
            None => detail = detail.push(text(t::TESTS_WAHL).size(12).color(pal.mute)),
            Some((g, it)) => {
                let o = &it.outcome;
                detail = detail.push(text(o.name.clone()).size(13.5).color(pal.text));
                detail = detail.push(text(testlauf::zustand_text(o)).size(12).color(farbe(o)));
                if !o.detail.is_empty() { detail = detail.push(text(o.detail.clone()).size(12).color(pal.text)) }
                if let Some(m) = &o.measured { detail = detail.push(text(format!("gemessen: {m}")).size(11.5).font(mono).color(pal.accent)) }
                if let Some(r) = &o.regs { detail = detail.push(text(format!("A {:02X}  X {:02X}  Y {:02X}  SP {:02X}  PC {:04X}", r.a, r.x, r.y, r.sp, r.pc)).size(11.5).font(mono).color(pal.text)); detail = detail.push(text("NV-BDIZC".chars().enumerate().map(|(i, ch)| if r.flags & (0x80 >> i) != 0 { ch } else { '·' }).collect::<String>()).size(11.5).font(mono).color(pal.ok)) }
                if g.file.is_some() { if let Some(s) = testlauf::TestLauf::setup_zeile(&it.file, &o.name) { detail = detail.push(text(s).size(11).font(mono).color(pal.mute)) } }
                let mut knoepfe = row![].spacing(6);
                if let Some((d, z)) = &o.location { let rel = vc_ide::tests::suite::rel_to(&self.root, d); knoepfe = knoepfe.push(knopf_k(format!("Zur Quelle ({rel}:{z})"), Some(Msg::TestQuelle(rel, *z)))) }
                let emu = tl.backend.is_some(); let kann = g.file.is_some() && !g.mailbox && emu && !tl.laeuft() && o.state != State::Skipped;
                knoepfe = knoepfe.push(knopf_k(t::TEST_DEBUG.into(), kann.then(|| Msg::TestDebug(it.file.clone(), o.name.clone()))));
                detail = detail.push(knoepfe);
            }
        }
        column![container(leiste).padding([4, 8]), row![scrollable(liste.padding([0, 8])).width(Length::FillPortion(3)).height(Length::Fill), container(scrollable(detail.padding(8))).width(Length::FillPortion(2)).height(Length::Fill).style(move |_| flaeche(pal.panel2, pal.line))]].into()
    }
    /// Reiter der Schublade in der Bereichskopfzeile (ein Klick auf einen Reiter klappt sie auch auf)
    fn schublade_kopf(&self, pal: Pal) -> Element<'_, Msg> {
        let tests_titel = match self.tests.status_text() { Some(s) => s, None => t::TESTS.to_string() };
        let mut reiter = row![].spacing(6);
        if self.dbg.is_some() { reiter = reiter.push(reiter_knopf(pal, t::DEBUG_REITER, self.schublade == 3, Msg::Schublade(3))) }
        reiter = reiter.push(reiter_knopf(pal, t::PROTOKOLL, self.schublade == 0, Msg::Schublade(0))).push(reiter_knopf(pal, &format!("{} ({})", t::PROBLEME, self.probleme.len()), self.schublade == 1, Msg::Schublade(1))).push(reiter_knopf(pal, &tests_titel, self.schublade == 2, Msg::Schublade(2)));
        bw::kopfzeile(pal, t::SCHUBLADE, self.layout.anzeige(Bereich::Schublade), reiter.wrap().vertical_spacing(2).into(), true, "Cmd+J", KUERZEL_MAX, akt(Bereich::Schublade))
    }
    /// Schublade minimiert: eine Leiste mit den Reitern
    fn schublade_leiste(&self, pal: Pal) -> Element<'_, Msg> { container(self.schublade_kopf(pal)).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into() }
    /// Linke Spalte minimiert: schmaler Streifen mit den Symbolen für Dateien und Suche
    fn spalten_streifen(&self, pal: Pal) -> Element<'_, Msg> {
        let k = |symbol: &'static str, s: Seite| -> Element<Msg> { button(text(symbol).size(15).color(pal.mute).center()).width(30).height(30).on_press(Msg::SpalteAuf(s)).style(move |_, st| button::Style { background: matches!(st, button::Status::Hovered).then(|| pal.panel2.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() };
        container(column![k("≡", Seite::Dateien), k("⌕", Seite::Suche)].spacing(4).align_x(Alignment::Center)).width(38).height(Length::Fill).padding([8, 4]).style(move |_| flaeche(pal.panel, pal.line)).into()
    }
    fn seitenleiste(&self, pal: Pal, breit: bool) -> Element<'_, Msg> {
        let umschalter = row![reiter_knopf(pal, t::DATEIEN, self.seite == Seite::Dateien, Msg::SeiteWahl(Seite::Dateien)), reiter_knopf(pal, t::SUCHE, self.seite == Seite::Suche, Msg::SeiteWahl(Seite::Suche))].spacing(4);
        let umschalter = bw::kopfzeile(pal, t::SPALTE, self.layout.anzeige(Bereich::Seitenleiste), umschalter.into(), true, "Cmd+B", KUERZEL_MAX, akt(Bereich::Seitenleiste));
        let aktiv = self.name();
        let inhalt: Element<Msg> = match self.seite {
            Seite::Dateien => {
                let offen = self.offene_dateien(); let start = project::startprogramm(&self.proj);
                let zeilen = project::baum_zeilen(&self.proj, &self.baum_zu, &self.baum_filter, &self.exportiert);
                let (n_dat, n_ord) = (zeilen.iter().filter(|z| !z.ordner).count(), zeilen.iter().filter(|z| z.ordner).count());
                let mut c = column![].spacing(1);
                for z in zeilen {
                    let einzug = iced::widget::space::horizontal().width(z.tiefe as f32 * 14.0);
                    let rel = z.rel.clone();
                    if z.ordner {
                        let enthaelt = !z.offen && offen.iter().any(|o| o.starts_with(&format!("{}/", z.rel)));
                        let marke = text(if z.offen { "▾" } else { "▸" }).size(11).color(pal.mute); let r2 = rel.clone();
                        let hinweis: Element<Msg> = if enthaelt { iced::widget::tooltip(text("●").size(9).color(pal.accent), container(text("enthält geöffnete Dateien").size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Right).into() } else { space::horizontal().width(0).into() };
                        let ziel_hier = self.baum_zug.as_ref().is_some_and(|g| g.bewegt && g.ziel.as_deref() == Some(rel.as_str())); let kb_fokus = self.fokus_bereich == Bereich::Seitenleiste && self.baum_sel.as_deref() == Some(rel.as_str());
                        let zeile = button(row![einzug, marke, text(format!("{}/", z.name)).size(13).color(pal.text), hinweis].spacing(5).align_y(Alignment::Center)).width(Length::Fill).padding([3, 6]).style(move |_, _| button::Style { background: ziel_hier.then(|| pal.sel.into()), border: Border { radius: 6.0.into(), color: if ziel_hier || kb_fokus { pal.accent } else { Color::TRANSPARENT }, width: if kb_fokus { 2.0 } else if ziel_hier { 1.0 } else { 0.0 } }, text_color: pal.text, ..Default::default() });
                        c = c.push(kontext::rechtsklick(self.zug_zeile(zeile.into(), &rel, true), move |pt| Msg::KontextAuf(Art::Baum, pt, Some(r2.clone()))));
                    } else {
                        // aktiv = aktuelle Datei der aktiven Gruppe (kräftig: Füllung und Akzentbalken); geöffnet = in einer Gruppe als Reiter offen (leichte Füllung und Punkt)
                        let kb_fokus = self.fokus_bereich == Bereich::Seitenleiste && self.baum_sel.as_deref() == Some(z.rel.as_str()); let aktuell = aktiv.as_deref() == Some(z.rel.as_str()); let geoeffnet = !aktuell && offen.contains(&z.rel); let r2 = rel.clone();
                        let (tag, farbe) = z.gruppe.map_or(("", pal.mute), |g| (g.tag(), pal.art_farbe(g)));
                        let geaendert = self.ist_geaendert(&z.rel); let ist_start = start.as_deref() == Some(z.rel.as_str());
                        let balken = container(space::vertical()).width(3).height(18).style(move |_| container::Style { background: if aktuell { Some(pal.accent.into()) } else if geoeffnet { Some(pal.mute.into()) } else { None }, border: Border { radius: 2.0.into(), ..Border::default() }, ..Default::default() });
                        let mut zeile = row![balken, einzug, text(tag).size(9.5).color(farbe).width(30), text(z.name.clone()).size(13).color(pal.text), space::horizontal()].spacing(4).align_y(Alignment::Center);
                        if geaendert { zeile = zeile.push(text("●").size(10).color(pal.warn)) } if ist_start { zeile = zeile.push(text("★").size(11).color(pal.accent)) }
                        c = c.push(kontext::rechtsklick(self.zug_zeile(button(zeile).width(Length::Fill).padding([3, 4]).style(move |_, _| button::Style { background: if aktuell { Some(pal.sel.into()) } else if geoeffnet { Some(pal.panel2.into()) } else { None }, border: Border { radius: 6.0.into(), color: if aktuell || kb_fokus { pal.accent } else { Color::TRANSPARENT }, width: if kb_fokus { 2.0 } else if aktuell { 1.0 } else { 0.0 } }, text_color: pal.text, ..Default::default() }).into(), &rel, false), move |pt| Msg::KontextAuf(Art::Baum, pt, Some(r2.clone()))));
                    }
                }
                if self.proj.entries.is_empty() && self.proj.dirs.is_empty() { c = c.push(text("Das Projekt ist leer.").size(12).color(pal.mute)) }
                let sk = |t: &'static str, tipp: &'static str, m: Msg| -> Element<Msg> { iced::widget::tooltip(button(text(t).size(13).color(pal.text)).padding([3, 8]).on_press(m).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }), container(text(tipp).size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Bottom).into() };
                let aktionen = row![text_input("Dateien filtern", &self.baum_filter).id("baumfilter").on_input(Msg::BaumFilter).padding(5).size(12.5), sk("＋", "Neue Datei im gewählten Ordner", Msg::BaumNeu(self.baum_ziel.clone(), NeuArt::Asm)), sk("⟳", "Baum neu einlesen", Msg::BaumNeuLaden), sk("⊟", "Alle Ordner zuklappen", Msg::BaumAlleZu)].spacing(4).align_y(Alignment::Center);
                let fuss = text(format!("{n_dat} Datei(en) · {n_ord} Ordner{}", if self.baum_filter.trim().is_empty() { "" } else { " (gefiltert)" })).size(11).color(pal.mute);
                let c = column![aktionen, scrollable(c).height(Length::Fill), fuss].spacing(6);
                container(c).height(Length::Fill).into()
            }
            Seite::Suche => {
                let mut c = column![text_input(t::SUCHE_PLATZHALTER, &self.suche_frage).id("suche").on_input(Msg::SucheFrage).on_submit(Msg::SucheLos).padding(6)].spacing(6);
                if self.sucht { c = c.push(text("Suche läuft …").size(12).color(pal.mute)) } else if self.suche_fertig { c = c.push(text(if self.funde.is_empty() { t::SUCHE_LEER.to_string() } else { format!("{} {}", self.funde.len(), t::TREFFER) }).size(12).color(pal.mute)) }
                for (i, f) in self.funde.iter().enumerate().take(300) { c = c.push(button(column![text(format!("{}:{}", f.rel, f.zeile)).size(11).color(pal.mute), text(f.text.clone()).size(12).font(vc_gui_ui::fonts::MONO).color(pal.text)].spacing(1)).width(Length::Fill).padding([3, 6]).on_press(Msg::GotoFund(i)).style(move |_, _| button::Style { background: None, text_color: pal.text, ..Default::default() })) }
                scrollable(c).height(Length::Fill).into()
            }
        };
        // Ohne Knopfzeile: der Baum nimmt die freie Höhe; Neues, Umbenennen, Löschen stehen im Kontextmenü (Rechtsklick), in der Palette und im Menü
        let inhalt: Element<Msg> = if self.seite == Seite::Dateien { kontext::rechtsklick(container(inhalt).width(Length::Fill).height(Length::Fill), |pt| Msg::KontextAuf(Art::BaumFlaeche, pt, Some(String::new()))) } else { inhalt };
        container(column![umschalter, inhalt].spacing(8)).width(if breit { Length::Fill } else { Length::Fixed(260.0) }).height(Length::Fill).padding(10).style(move |_| flaeche(pal.panel, pal.line)).into()
    }
    /// Dialog des Dateibaums (Neue Datei, Neuer Ordner, Umbenennen, Löschen) als Karte über der Oberfläche
    /// Rückfrage „Eine Debug-Sitzung läuft. Beenden und …?“ mit [Sitzung beenden] [Abbrechen]
    fn rueckfrage_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let r = self.rueckfrage.as_ref()?;
        let (titel, frage, ja) = match r { Rueckfrage::TestDebug(_, n) => (t::RQ_TITEL, format!("Eine Debug-Sitzung läuft. Beenden und den Test „{n}“ debuggen?"), t::RQ_BEENDEN), Rueckfrage::DebugNeu => (t::RQ_TITEL, "Eine Debug-Sitzung läuft. Beenden und neu bauen und debuggen?".to_string(), t::RQ_BEENDEN), Rueckfrage::ScratchWeg(n) => ("Reiter schließen", format!("„{n}“ hat Inhalt und ist nicht gespeichert (Cmd+S legt ihn als Datei an). Verwerfen?"), "Verwerfen"), Rueckfrage::Kachelkarte(n) => ("Ältere Kachelkarte umwandeln?", format!("„{n}“ ist eine ältere Kachelkarte (kind „tilemap“). Das Kachelsatz-Studio liest Kachelsatz und Bildschirm: Umwandeln legt beides neben der Datei an und öffnet den Bildschirm. Die alte Datei wird vorher unter .vcbackup/ gesichert und dann durch die neuen ersetzt."), "Umwandeln"), Rueckfrage::KonfigImport(p, z) => ("Konfiguration übernehmen?", format!("Die Datei „{}“ enthält: {z}. Die jetzigen Einstellungen für Zielsysteme, Emulatoren, Maschinen und Parameter werden ersetzt; bekannte Maschinen behalten ihr Passwort. Die Oberflächeneinstellungen (Farbschema, Zoom) bleiben.", p.display()), "Übernehmen"), Rueckfrage::KonfigReset => ("Einstellungen zurücksetzen?", "Alle Einstellungen der IDE (Zielsysteme, Emulatoren, Maschinen samt Passwörtern, Pfade, Parameter) gehen auf die Vorgabe zurück. Das lässt sich nicht rückgängig machen; vorher kann man die Konfiguration exportieren.".to_string(), "Zurücksetzen") };
        let c = column![text(titel).size(16).color(pal.text), text(frage).size(12.5).color(pal.text), row![knopf(pal, ja, Some(Msg::RueckfrageJa), pal.text, true), knopf(pal, t::BAUM_ABBRUCH, Some(Msg::RueckfrageNein), pal.text, false)].spacing(8)].spacing(12);
        let karte = container(c).width(440).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hintergrund = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() })).on_press(Msg::RueckfrageNein);
        Some(stack![hintergrund, container(karte).center(Length::Fill)].into())
    }
    /// Zeile „Ordner“ der Anlege-Dialoge: Auswahlliste aller Projektordner, `(Projektordner)` = Wurzel
    fn ordner_wahl(&self, pal: Pal, aktuell: &str) -> Element<'_, Msg> {
        let mut liste = vec![OrdnerWahl(String::new())]; liste.extend(self.proj.dirs.iter().filter(|d| !d.split('/').any(|t| t.starts_with('.'))).map(|d| OrdnerWahl(d.clone())));
        liste.sort(); liste.dedup();
        let wahl = OrdnerWahl(aktuell.to_string()); if !liste.contains(&wahl) { liste.push(wahl.clone()) }
        column![text("Ordner").size(11).color(pal.mute), iced::widget::pick_list(liste, Some(wahl), |o| Msg::BaumOrdnerWahl(o.0)).text_size(12.5).padding(6).width(Length::Fill)].spacing(4).into()
    }
    fn baum_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let b = self.baum.as_ref()?;
        let titel = match &b.art { BaumArt::Neu { .. } => t::DLG_NEUE_DATEI, BaumArt::Ordner(_) => t::DLG_NEUER_ORDNER, BaumArt::Umbenennen(_) => t::DLG_UMBENENNEN, BaumArt::Loeschen(_) => t::DLG_LOESCHEN, BaumArt::SpeichernUnter(_) => "Speichern unter" };
        let mut c = column![text(titel).size(16).color(pal.text)].spacing(10);
        match &b.art {
            BaumArt::Loeschen(rel) => { c = c.push(text(format!("{rel} {}", if self.papierkorb { t::BAUM_FRAGE_KORB } else { t::BAUM_FRAGE_ENDGUELTIG })).size(12.5).color(pal.text)) }
            art => {
                if let BaumArt::Neu { ordner, art: gewaehlt, profile, profil } = art {
                    let mut arten = vec![NeuArt::Asm, NeuArt::Basic, NeuArt::Text, NeuArt::Notiz]; arten.extend(self.studio_arten().into_iter().map(NeuArt::Studio)); arten.extend([NeuArt::Diskette("d64"), NeuArt::Diskette("d71"), NeuArt::Diskette("d81"), NeuArt::Diskette("atr")]);
                    c = c.push(text(t::DLG_ART).size(11).color(pal.mute));
                    let mut zeile = row![].spacing(4); let mut zeilen = column![].spacing(4); let mut n = 0;
                    for a in arten { let on = a == *gewaehlt; zeile = zeile.push(button(text(a.name()).size(12.5).color(pal.text)).padding([4, 9]).on_press(Msg::BaumArtWahl(a)).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() })); n += 1; if n % 4 == 0 { zeilen = zeilen.push(std::mem::replace(&mut zeile, row![].spacing(4))) } }
                    zeilen = zeilen.push(zeile); c = c.push(zeilen);
                    if let NeuArt::Studio(_) = gewaehlt { c = c.push(text(t::DLG_PROFIL).size(11).color(pal.mute)); for (i, (_, name)) in profile.iter().enumerate() { let on = i == *profil; c = c.push(button(text(format!("{} {name}", if on { "●" } else { "○" })).size(12.5).color(pal.text)).padding([3, 6]).on_press(Msg::BaumProfil(i)).style(move |_, _| button::Style { background: None, text_color: pal.text, ..Default::default() })) } }
                    c = c.push(self.ordner_wahl(pal, ordner));
                } else if let BaumArt::Ordner(ordner) = art { c = c.push(self.ordner_wahl(pal, ordner)); }
                c = c.push(text(t::DLG_NAME).size(11).color(pal.mute));
                c = c.push(text_input(t::BAUM_PFAD, &b.text).id("baum").on_input(Msg::BaumText).on_submit(Msg::BaumOk).padding(7));
                // Vorschau des endgültigen Pfads, mit der Endung, die die Art ergänzt (wie die Endungs-Automatik beim Speichern im Betriebssystem)
                match &b.art { BaumArt::Neu { ordner, art: ga, .. } if !b.text.trim().is_empty() => { let n = b.text.trim().to_string(); let n = if n.contains('.') || matches!(ga, NeuArt::Studio(_)) { n } else { format!("{n}{}", ga.endung()) }; let rel = if ordner.is_empty() { n } else { format!("{ordner}/{n}") }; c = c.push(text(format!("Wird angelegt: {rel}")).size(11.5).color(pal.mute)) } _ => {} }
            }
        }
        if !b.fehler.is_empty() { c = c.push(text(b.fehler.clone()).size(12).color(pal.err)) }
        c = c.push(row![knopf(pal, t::BAUM_OK, Some(Msg::BaumOk), pal.text, true), knopf(pal, t::BAUM_ABBRUCH, Some(Msg::BaumAbbruch), pal.text, false)].spacing(8));
        let karte = container(c).width(440).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, shadow: iced::Shadow { color: Color { a: 0.45, ..Color::BLACK }, offset: iced::Vector::new(0.0, 6.0), blur_radius: 18.0 }, ..Default::default() });
        let hintergrund = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() })).on_press(Msg::BaumAbbruch);
        Some(stack![hintergrund, container(karte).center(Length::Fill)].into())
    }
    fn panel(&self, pal: Pal, breit: bool) -> Element<'_, Msg> {
        let reiter = |name: &'static str, p: PanelSeite| -> Element<Msg> { let on = self.panel_seite == p; button(text(name).size(12).color(if on { pal.text } else { pal.mute })).padding([4, 8]).on_press(Msg::PanelWahl(p)).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel.into() }), border: Border { radius: 4.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() };
        let mut kopf = row![reiter(t::GLIEDERUNG, PanelSeite::Gliederung), reiter(t::HILFE_TITEL, PanelSeite::Hilfe), reiter(t::ZEICHEN, PanelSeite::Zeichen)].spacing(4);
        kopf = kopf.push(reiter(t::KARTE, PanelSeite::Karte));
        if self.dbg.is_some() && self.snap.can_frame { kopf = kopf.push(reiter(t::EMULATOR_BILD, PanelSeite::Emulator)) }
        let kopf = bw::kopfzeile(pal, t::PANEL, self.layout.anzeige(Bereich::Panel), kopf.wrap().vertical_spacing(2).into(), true, "Cmd+Alt+B", KUERZEL_MAX, akt(Bereich::Panel));
        let inhalt = match self.panel_seite { PanelSeite::Gliederung => self.panel_gliederung(pal), PanelSeite::Hilfe => self.panel_hilfe(pal), PanelSeite::Zeichen => self.panel_zeichen(pal), PanelSeite::Debug => self.panel_debug(pal), PanelSeite::Karte => self.panel_karte(pal), PanelSeite::Emulator => self.panel_emulator(pal) };
        container(column![kopf, inhalt].spacing(8)).width(if breit { Length::Fill } else { Length::Fixed(self.panel_breite()) }).height(Length::Fill).padding(10).style(move |_| flaeche(pal.panel, pal.line)).into()
    }
    fn zeile_knopf<'a>(pal: Pal, inhalt: Element<'a, Msg>, m: Msg) -> Element<'a, Msg> { button(inhalt).width(Length::Fill).padding([4, 6]).on_press(m).style(move |_, st| button::Style { background: matches!(st, button::Status::Hovered).then(|| pal.sel.into()), text_color: pal.text, border: Border { radius: 4.0.into(), ..Border::default() }, ..Default::default() }).into() }
    /// Panel „Speicherkarte“: 256 Seiten (16 × 16), Bankumschaltung, Programmbelegung, Auswahl mit Einzelheiten
    fn panel_karte(&self, pal: Pal) -> Element<'_, Msg> {
        use vc_ide_model::memmap_facts as mm; use vc_target::{Page, PageCategory};
        let Some(info) = &self.info else { return container(text(t::KARTE_LEER).size(12.5).color(pal.mute)).padding(8).into() };
        let k = &self.karte; let facts = k.facts.clone().unwrap_or_default();
        let preset = k.banking.as_ref().and_then(|b| k.preset.as_deref().and_then(|i| b.preset(i)).or_else(|| b.default_preset()));
        let plan: [Page; 256] = match (&k.banking, preset) { (Some(b), Some(p)) => b.page_plan(p), _ => mm::plain_plan() };
        let usage = mm::page_usage(&facts.segs); let filter = mm::Filter::parse(k.filter);
        let farbe = move |c: PageCategory| -> Color { let (d, h) = match c { PageCategory::Free => ((0x2a, 0x2e, 0x36), (0xe4, 0xe7, 0xec)), PageCategory::Zeropage => ((0x2a, 0x8f, 0xb4), (0x7f, 0xc4, 0xdf)), PageCategory::Stack => ((0x1f, 0x9d, 0x8f), (0x72, 0xcc, 0xc0)), PageCategory::Screen => ((0xb4, 0x4f, 0x8a), (0xe0, 0x9c, 0xc0)), PageCategory::Sprites => ((0xb8, 0x55, 0x5f), (0xe3, 0xa0, 0xa6)), PageCategory::Charset => ((0x2f, 0x8f, 0x5a), (0x8f, 0xd0, 0xa8)), PageCategory::Music => ((0x9a, 0x86, 0x2b), (0xd6, 0xc4, 0x78)), PageCategory::Io => ((0xb0, 0x6c, 0x2e), (0xe0, 0xb0, 0x80)), PageCategory::Rom => ((0x6b, 0x66, 0x7c), (0xb4, 0xb0, 0xc4)), PageCategory::Basic => ((0x7a, 0x4c, 0xb8), (0xbd, 0x9c, 0xe0)) }; let (r, g, b) = if pal.dark { d } else { h }; Color::from_rgb8(r, g, b) };
        let titel = |s: String| -> Element<Msg> { text(s).size(11).color(pal.mute).into() };
        let mono = |s: String, c: Color| -> Element<Msg> { text(s).size(11.5).font(vc_gui_ui::fonts::MONO).color(c).into() };
        let filter_zeile = row(["all", "prog", "ram", "rom", "io"].iter().zip(["Alle", "Programm", "RAM", "ROM", "E/A"]).map(|(f, n)| { let on = k.filter == *f; let f: &'static str = f; button(text(n).size(11).color(if on { pal.text } else { pal.mute })).padding([2, 6]).on_press(Msg::KarteFilter(f)).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { radius: 4.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(4);
        let mut c = column![titel(format!("{} · 64 KB", info.target.to_uppercase())), filter_zeile].spacing(6);
        // Matrix: 16 Zeilen à 16 Seiten; Programmbelegung als unterer Balken in Cyan
        let mut gitter = column![].spacing(1);
        for r in 0..16usize {
            let mut z = row![container(text(format!("${r:X}000")).size(9).font(vc_gui_ui::fonts::MONO).color(pal.mute)).width(28).align_right(Length::Fill)].spacing(1).align_y(Alignment::Center);
            for sp in 0..16usize {
                let p = r * 16 + sp; let (pg, u) = (plan[p], usage[p]); let dim = !filter.matches(pg.kind, u); let auswahl = k.sel as usize == p;
                let basis = farbe(pg.category); let basis = if dim { Color { a: 0.3, ..basis } } else { basis };
                let zelle = container(space::vertical()).width(15).height(15).style(move |_| container::Style { background: Some(basis.into()), border: Border { color: if auswahl { pal.text } else { pal.line }, width: if auswahl { 2.0 } else { 0.5 }, radius: 2.0.into() }, ..Default::default() });
                let belegt = (u.min(256) as f32 / 256.0 * 15.0).round().max(if u > 0 { 2.0 } else { 0.0 });
                let inhalt: Element<Msg> = if u > 0 { iced::widget::stack![zelle, container(container(space::vertical()).width(15).height(belegt).style(move |_| container::Style { background: Some(Color { a: if dim { 0.35 } else { 1.0 }, ..pal.accent }.into()), ..Default::default() })).height(15).align_bottom(15)].into() } else { zelle.into() };
                z = z.push(button(inhalt).padding(0).on_press(Msg::KarteSeite(p as u8)).style(move |_, _| button::Style { background: None, ..Default::default() }));
            }
            gitter = gitter.push(z);
        }
        c = c.push(gitter);
        if let Some(b) = &k.banking {
            c = c.push(titel(format!("{} (${:04X})", b.register.name, b.register.addr)));
            let sel_id = preset.map(|p| p.id.clone()).unwrap_or_default();
            for p in &b.presets { let on = p.id == sel_id; c = c.push(button(text(mm::preset_label(p)).size(11).color(if on { pal.text } else { pal.mute })).width(Length::Fill).padding([2, 6]).on_press(Msg::KartePreset(p.id.clone())).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel.into() }), border: Border { radius: 4.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() })) }
            if let Some(p) = preset { if !p.note.is_empty() { c = c.push(text(p.note.clone()).size(11).color(pal.mute)) } }
        }
        // Legende
        c = c.push(row(mm::legend(&plan).into_iter().map(|cat| row![container(space::vertical()).width(8).height(8).style(move |_| container::Style { background: Some(farbe(cat).into()), ..Default::default() }), text(mm::category_label(cat)).size(10).color(pal.mute)].spacing(3).align_y(Alignment::Center).into()).chain(std::iter::once(row![container(space::vertical()).width(8).height(8).style(move |_| container::Style { background: Some(pal.accent.into()), ..Default::default() }), text("Programm").size(10).color(pal.mute)].spacing(3).align_y(Alignment::Center).into())).collect::<Vec<_>>()).spacing(8).wrap());
        // Auswahl
        let p = k.sel; let pg = plan[p as usize];
        c = c.push(titel(format!("SEITE ${p:02X} · ${:04X}–${:04X}", (p as u32) << 8, ((p as u32) << 8) + 255)));
        c = c.push(mono(format!("{} · {}", mm::kind_label(pg.kind), mm::category_label(pg.category)), pal.text));
        if usage[p as usize] > 0 { c = c.push(mono(format!("Programm: {} von 256 Byte", usage[p as usize]), pal.accent)) }
        for (n, a) in mm::page_symbols(&facts, p, p).into_iter().take(8) { c = c.push(mono(format!("{n}  ${a:04X}"), pal.lab)) }
        for (f, a, b) in mm::page_sources(&info.zeilen, p, p).into_iter().take(4) { c = c.push(mono(format!("{f}:{a}{}", if b != a { format!("–{b}") } else { String::new() }), pal.mute)) }
        // Prüfungen
        c = c.push(titel("PRÜFUNG".into()));
        for ch in mm::checks(&facts, Some(&plan)) { c = c.push(mono(format!("{} {}", match ch.level { mm::Level::Ok => "✓", mm::Level::Info => "·", mm::Level::Warn => "⚠", mm::Level::Err => "✗" }, ch.text), match ch.level { mm::Level::Err => pal.err, mm::Level::Warn => pal.warn, _ => pal.mute })) }
        scrollable(c.spacing(4).padding(iced::Padding { right: 12.0, left: 2.0, ..Default::default() })).height(Length::Fill).into()
    }
    /// Panel „Emulator“: Bild des angehaltenen (oder in Zeitlupe laufenden) Emulators, Tastatureingabe
    fn panel_emulator(&self, pal: Pal) -> Element<'_, Msg> {
        let sn = &self.snap;
        let knopf_e = |name: &'static str, m: Msg| -> Element<Msg> { button(text(name).size(11.5).color(pal.text)).padding([3, 8]).on_press(m).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }).into() };
        let mut c = column![row![knopf_e(t::EMU_AKTUALISIEREN, Msg::EmuBild),
            iced::widget::checkbox(self.emu_auto).label(t::EMU_AUTO).on_toggle(Msg::EmuAuto).size(14).text_size(11.5),
            iced::widget::checkbox(self.emu_live).label(t::EMU_LIVE).on_toggle(Msg::EmuLive).size(14).text_size(11.5)].spacing(10).align_y(Alignment::Center)].spacing(8);
        if sn.frame_sources.len() > 1 { c = c.push(row(sn.frame_sources.iter().enumerate().map(|(i, n)| button(text(n.clone()).size(11.5).color(pal.text)).padding([2, 8]).on_press(Msg::EmuQuelle(i)).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { radius: 4.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into()).collect::<Vec<_>>()).spacing(4)) }
        match &self.emu_bild {
            Some((_, h, w, hh)) => {
                c = c.push(container(image(h.clone()).width(Length::Fill).content_fit(iced::ContentFit::Contain).filter_method(image::FilterMethod::Nearest)).width(Length::Fill).style(move |_| container::Style { background: Some(Color::BLACK.into()), ..Default::default() }));
                c = c.push(text(format!("{w}×{hh} · Bild {} · Aufbau {:.2} ms", sn.frame_seq, self.emu_ms)).size(11).font(vc_gui_ui::fonts::MONO).color(pal.mute));
            }
            None => c = c.push(text(if sn.running { t::EMU_LAEUFT } else { t::EMU_KEIN_BILD }).size(12).color(pal.mute)),
        }
        if let Some(e) = &sn.frame_err { c = c.push(text(debug::freundlich(e)).size(11.5).color(pal.warn)) }
        if sn.can_text { c = c.push(text_input(t::EMU_TEXT, &self.emu_text).on_input(Msg::EmuText).on_submit(Msg::EmuSenden).size(12).padding(4)) }
        scrollable(c.padding(iced::Padding { right: 12.0, ..Default::default() })).height(Length::Fill).into()
    }
    /// Titel eines Hilfe-Reiters (Thema oder „Hilfe“)
    fn hilfe_titel(&self, k: &str) -> String { if k.is_empty() { "Hilfe".into() } else { vc_ide_model::helpstate::open(&self.data, &self.hilfe_ziel(), self.hilfe_sprache(), k).map(|d| d.title).unwrap_or_else(|| k.to_string()) } }
    /// Hilfe-Thema im Dokumentenfeld: gerendertes Markdown in lesbarer Breite, Links zwischen Themen wirken im selben Reiter
    fn hilfe_reiter(&self, pal: Pal, k: &str) -> Element<'_, Msg> {
        let inhalt: Element<Msg> = match if k.is_empty() { None } else { vc_ide_model::helpstate::open(&self.data, &self.hilfe_ziel(), self.hilfe_sprache(), k) } {
            Some(d) => { let doc = self.md_doc(&d.markdown); container(mdansicht::ansicht(pal, &doc, usize::MAX, None)).max_width(760).into() }
            None => { let mut c = column![text("Hilfe").size(21).font(vc_gui_ui::fonts::UI_FETT).color(pal.text)].spacing(6); let mut gruppe = String::new();
                for th in &self.hilfe.themen { if th.group != gruppe { gruppe = th.group.clone(); c = c.push(text(gruppe.to_uppercase()).size(10.5).color(pal.mute)) } c = c.push(Self::zeile_knopf(pal, text(th.title.clone()).size(13).color(pal.text).into(), Msg::MdLink(format!("help:{}", th.id)))) }
                container(c).max_width(520).into() }
        };
        container(scrollable(container(inhalt).padding(iced::Padding { top: 16.0, right: 28.0, bottom: 28.0, left: 28.0 })).height(Length::Fill)).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).into()
    }
    fn panel_hilfe(&self, pal: Pal) -> Element<'_, Msg> {
        use vc_ide_model::helpstate::Line;
        let suche = text_input(t::HILFE_SUCHE, &self.hilfe.suche).id("hilfesuche").on_input(Msg::HilfeSuche).padding(6);
        let inhalt: Element<Msg> = if let Some(d) = &self.hilfe.doc {
            let doc = self.md_doc(&d.markdown);
            let kopf = row![button(text(t::HILFE_ZURUECK).size(12).color(pal.accent)).padding([2, 4]).on_press(Msg::HilfeZurueck).style(|_, _| button::Style { background: None, ..Default::default() }), space::horizontal(), vc_gui_ui::muster::knopf(pal, "Im Reiter öffnen", "Dieses Thema im Dokumentenfeld öffnen (mehr Platz)", Some(Msg::HilfeImReiter), false, false)].align_y(Alignment::Center);
            let c = column![kopf, container(mdansicht::ansicht_schmal(pal, &doc, usize::MAX, None)).max_width(620)].spacing(8);
            scrollable(c.padding(iced::Padding { left: 6.0, right: 16.0, top: 0.0, bottom: 8.0 })).height(Length::Fill).into()
        } else if !self.hilfe.suche.trim().is_empty() {
            if self.hilfe.treffer.is_empty() { text(t::HILFE_NICHTS).size(12.5).color(pal.mute).into() } else {
                scrollable(column(self.hilfe.treffer.iter().take(60).map(|r| Self::zeile_knopf(pal, column![text(r.title.clone()).size(13).color(pal.text), text(r.snippet.clone()).size(11).color(pal.mute)].spacing(1).into(), Msg::HilfeOeffne(r.id.clone()))).collect::<Vec<_>>()).spacing(2)).height(Length::Fill).into() }
        } else {
            let mut c = column![].spacing(2); let mut gruppe = String::new();
            for th in &self.hilfe.themen { if th.group != gruppe { gruppe = th.group.clone(); c = c.push(text(gruppe.to_uppercase()).size(10.5).color(pal.mute)) } c = c.push(Self::zeile_knopf(pal, text(th.title.clone()).size(13).color(pal.text).into(), Msg::HilfeOeffne(th.id.clone()))) }
            scrollable(c).height(Length::Fill).into()
        };
        column![suche, inhalt].spacing(8).into()
    }
    fn panel_zeichen(&self, pal: Pal) -> Element<'_, Msg> {
        use vc_ide_model::charpanel::{self, Hit};
        let lang = self.name().and_then(|n| self.editoren.get(&n)).map_or(vc_ide_model::lang::Lang::Asm, |e| e.lang);
        let list = self.zeichen_eintraege(lang); let atari = self.atari();
        let treffer: Vec<Hit> = if self.zeichen_suche.trim().is_empty() { (0..list.len()).map(Hit::Control).collect() } else { charpanel::search(&list, atari, &self.zeichen_suche) };
        let mut c = column![].spacing(2);
        for h in treffer.into_iter().take(200) { match h {
            Hit::Control(i) => if let Some(e) = list.get(i) { c = c.push(Self::zeile_knopf(pal, column![row![text(format!("{{{}}}", e.name)).size(13).font(vc_gui_ui::fonts::MONO).color(pal.lab), space::horizontal(), text(format!("${:02x}", e.code)).size(11).color(pal.mute)], text(e.desc.clone()).size(11).color(pal.mute)].spacing(1).into(), Msg::ZeichenEinfuegen(ZeichenWahl::Steuer(i)))) },
            Hit::Code(code) => { let lit = vc_ide_model::chartext::literal_of(atari, 0, code).map(|c| c.to_string()).unwrap_or_else(|| "·".into()); c = c.push(Self::zeile_knopf(pal, row![text(lit).size(13).font(vc_gui_ui::fonts::MONO).color(pal.lab), space::horizontal(), text(format!("${code:02x} · {code}")).size(11).color(pal.mute)].into(), Msg::ZeichenEinfuegen(ZeichenWahl::Code(code)))) }
        } }
        column![text_input(t::ZEICHEN_SUCHE, &self.zeichen_suche).id("zeichensuche").on_input(Msg::ZeichenSuche).padding(6), scrollable(container(c).padding(iced::Padding { right: 12.0, ..Default::default() })).height(Length::Fill)].spacing(8).into()
    }
    fn panel_gliederung(&self, pal: Pal) -> Element<'_, Msg> {
        let g = self.gliederung_holen();
        if g.is_empty() { text(t::GLIEDERUNG_LEER).size(12.5).color(pal.mute).into() } else {
            scrollable(column(g.into_iter().map(|(l, n)| { let datei = self.name().unwrap_or_default(); button(row![text(n).size(13).font(vc_gui_ui::fonts::MONO).color(pal.lab), space::horizontal(), text(format!("{}", l + 1)).size(11).color(pal.mute)]).width(Length::Fill).padding([3, 6]).on_press(Msg::GotoZeile(l)).style(move |_, _| { let _ = &datei; button::Style { background: None, text_color: pal.text, ..Default::default() } }).into() }).collect::<Vec<_>>()).spacing(1)).height(Length::Fill).into() }
    }
    fn palette_ansicht(&self, pal: Pal) -> Element<'_, Msg> {
        let ein = self.palette_eintraege();
        let liste = column(ein.iter().take(12).enumerate().map(|(i, e)| {
            let (name, tasten) = match e { Eintrag::Befehl(c) => { let inf = befehle::info(*c); (inf.name.to_string(), befehle::fuer_anzeige(inf.tasten, cfg!(target_os = "macos"))) } Eintrag::Datei(n) => (n.clone(), "Datei".into()), Eintrag::Projekt(_, n) => (format!("Projekt: {n}"), "Zuletzt geöffnet".into()), Eintrag::Studio(n, k, _) => (format!("Studio: {n}"), if k.is_empty() { "Studio".to_string() } else { k.clone() }) };
            let aus = matches!(e, Eintrag::Befehl(c) if !self.nutzbar(*c)); let sel = i == self.palette.auswahl;
            button(row![text(name).size(14).color(if aus { pal.mute } else { pal.text }), space::horizontal(), text(tasten).size(12).color(pal.mute)]).width(Length::Fill).padding([8, 12]).on_press_maybe((!aus).then_some(Msg::PaletteLos(i))).style(move |_, _| button::Style { background: sel.then(|| pal.sel.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>());
        let karte = container(column![text_input(t::SUCHFELD, &self.palette.frage).id("palette").on_input(Msg::PaletteFrage).on_submit(Msg::PaletteLos(self.palette.auswahl)).padding(10), liste].spacing(8).width(560)).padding(14).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 14.0.into() }, ..Default::default() });
        stack![container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Background::Color(Color { a: 0.5, ..Color::BLACK })), ..Default::default() }), container(karte).center_x(Length::Fill).padding(90)].into()
    }
    fn abo(&self) -> Subscription<Msg> {
        let mut v = vec![iced::event::listen_with(|e, status, _| fenster_ereignis(e, status))];
        if self.splash { v.push(iced::time::every(std::time::Duration::from_millis(2000)).map(|_| Msg::SplashZu)) }
        if !self.scratch.is_empty() { v.push(iced::time::every(std::time::Duration::from_secs(3)).map(|_| Msg::ScratchTick)) }
        if self.trenner { v.push(iced::event::listen_with(|e, _, _| match e {
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(Msg::TrennerMaus(position)),
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => Some(Msg::TrennerLos),
            _ => None })) }
        if self.ziehen.is_some() { v.push(iced::event::listen_with(|e, _, _| match e {
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(Msg::Maus(position)),
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => Some(Msg::MausLos),
            _ => None })) }
        if self.tests.laeuft() { v.push(iced::time::every(std::time::Duration::from_millis(150)).map(|_| Msg::TestTick)) }
        if self.name().is_some_and(|n| self.studios.get(&n).is_some_and(|s| s.doc.kind == "sprite" && s.doc.animationen().is_ok_and(|a| !a.is_empty()))) { v.push(iced::time::every(std::time::Duration::from_millis(40)).map(|_| Msg::StudioTick)) }
        if self.dbg.is_some() { v.push(iced::time::every(std::time::Duration::from_millis(100)).map(|_| Msg::DebugTick)) }
        if self.audio_song.is_some() || self.probe_aktiv { v.push(iced::time::every(std::time::Duration::from_millis(40)).map(|_| Msg::SongTick)) }
        // Ziehen im Dateibaum endet auch, wenn die Maus außerhalb der Zeilen losgelassen wird (dort nimmt keine Zeile das Loslassen an)
        if self.baum_zug.is_some() { v.push(iced::event::listen_with(|e, status, _| match e { iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) if status == iced::event::Status::Ignored => Some(Msg::BaumZugAbbruch), _ => None })) }
        v.push(Subscription::run_with(self.lsp_gen, |_| lsp_strom()));
        #[cfg(target_os = "macos")] v.push(native::menu_subscription());
        Subscription::batch(v)
    }
}

/// Leere Ordner unter `p` (und `p` selbst) entfernen, von unten nach oben; Ordner mit Inhalt bleiben
fn loesche_leere_ordner(p: &std::path::Path) {
    if let Ok(rd) = std::fs::read_dir(p) { for e in rd.flatten() { if e.path().is_dir() { loesche_leere_ordner(&e.path()) } } }
    let _ = std::fs::remove_dir(p);
}

fn lsp_strom() -> impl iced::futures::Stream<Item = Msg> {
    iced::stream::channel(32, async |mut out: iced::futures::channel::mpsc::Sender<Msg>| {
        use iced::futures::{SinkExt, StreamExt};
        let (tx, mut rx) = iced::futures::channel::mpsc::unbounded::<lsp::Antwort>();
        if let Some(r) = LSP_EMPFAENGER.lock().unwrap().take() { std::thread::spawn(move || { while let Ok(a) = r.recv() { if tx.unbounded_send(a).is_err() { break } } }); }
        while let Some(a) = rx.next().await { let _ = out.send(Msg::Lsp(a)).await; }
    })
}

/// Die Quelldatei, die beim Öffnen und Bauen gilt: erste .asm im Projektordner selbst, sonst irgendeine
fn hauptquelle(namen: &[String]) -> Option<String> { namen.iter().find(|n| n.ends_with(".asm") && !n.contains('/')).or_else(|| namen.iter().find(|n| n.ends_with(".asm"))).cloned() }

/// Buchstabe, der mit Alt das Menü öffnet (Mnemonic, unterstrichen); deutsche Windows-Gewohnheit: D Datei, B Bearbeiten, A Ansicht, P Projekt, H Hilfe
fn menue_buchstabe(menue: &str) -> Option<char> { match menue { "Datei" => Some('d'), "Bearbeiten" => Some('b'), "Ansicht" => Some('a'), "Projekt" => Some('p'), "Hilfe" => Some('h'), _ => None } }
/// Passt ein Kürzel („Cmd+Umschalt+F“, „F7“) zu dieser Taste? (Windows/Linux; auf macOS verarbeitet das Menü die Kürzel)
pub fn trifft(tasten: &str, k: &keyboard::Key, m: keyboard::Modifiers) -> bool {
    let Some((cmd, sh, alt, taste)) = befehle::zerlege(tasten) else { return false };
    if cmd != m.command() || sh != m.shift() || alt != m.alt() { return false }
    match k { keyboard::Key::Character(c) => { let c = c.as_str().to_uppercase(); match taste { "Plus" => c == "+" || c == "=", "Minus" => c == "-", "Komma" => c == ",", t => t == c } }
        keyboard::Key::Named(n) => { use keyboard::key::Named::*; matches!((taste, n), ("F1", F1) | ("F5", F5) | ("F6", F6) | ("F7", F7) | ("F8", F8) | ("F9", F9) | ("F10", F10) | ("F11", F11) | ("F12", F12) | ("Rechts", ArrowRight) | ("Links", ArrowLeft) | ("Enter", Enter)) }
        _ => false }
}

/// Datei im Dateimanager anzeigen (nur über `aussen`: im Testlauf wird nichts gestartet)
fn zeige_im_ordner(p: &std::path::Path) { aussen::ausfuehren(aussen::Aufruf::ImOrdnerZeigen(p.to_path_buf())) }
/// Markdown-Stücke (fett, Code) als Spannen eines Absatzes
fn md_spannen(pal: Pal, t: &str, groesse: f32, farbe: Color, fett: bool) -> Vec<iced::widget::text::Span<'static, (), iced::Font>> {
    let mut out = vec![]; let mut rest = t; let bold = vc_gui_ui::fonts::UI_FETT;
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix("**") { if let Some(e) = r.find("**") { out.push(iced::widget::span(r[..e].to_string()).font(bold).size(groesse).color(pal.text)); rest = &r[e + 2..]; continue } }
        if let Some(r) = rest.strip_prefix('`') { if let Some(e) = r.find('`') { out.push(iced::widget::span(format!(" {} ", &r[..e])).font(vc_gui_ui::fonts::MONO).size(groesse - 0.5).color(pal.lab).background(pal.editor)); rest = &r[e + 1..]; continue } }
        let ende = rest.char_indices().skip(1).find(|(_, c)| *c == '*' || *c == '`').map(|(i, _)| i).unwrap_or(rest.len());
        let (stueck, r) = rest.split_at(ende); out.push(iced::widget::span(stueck.to_string()).size(groesse).color(farbe).font(if fett { bold } else { iced::Font::DEFAULT })); rest = r;
    }
    out
}
/// Markdown (Überschrift in der ersten Zeile, Absätze, Aufzählungen, Tabellen) als Karte für Hinweise
fn md_karte(pal: Pal, md: &str) -> Element<'static, Msg> {
    let mut karte = column![].spacing(6); let mut erste = true; let mut zeilen = md.lines().peekable();
    while let Some(l) = zeilen.next() {
        let t = l.trim(); if t.is_empty() { continue }
        if t.starts_with('|') {
            // Tabelle: zusammenhängende Zeilen mit |; die Trennzeile (---) entfällt
            let mut tab: Vec<Vec<String>> = vec![t.trim_matches('|').split('|').map(|c| c.trim().to_string()).collect()];
            while let Some(n) = zeilen.peek() { if n.trim().starts_with('|') { let z = zeilen.next().unwrap().trim().trim_matches('|').to_string(); if !z.chars().all(|c| c == '-' || c == '|' || c == ':' || c == ' ') { tab.push(z.split('|').map(|c| c.trim().to_string()).collect()) } } else { break } }
            let spalten = tab.iter().map(|r| r.len()).max().unwrap_or(0);
            let breiten: Vec<f32> = (0..spalten).map(|i| tab.iter().filter_map(|r| r.get(i)).map(|c| c.replace('`', "").chars().count()).max().unwrap_or(1) as f32 * 7.9 + 18.0).collect(); let gesamt: f32 = breiten.iter().sum();
            let mut g = column![].spacing(0);
            for (zi, r) in tab.into_iter().enumerate() {
                let kopf = zi == 0; let mut zeile = row![].spacing(0);
                // Zeile mit „▶“: die vom Assembler gewählte Adressierungsart (Hover über einem Mnemonic) – Akzentfarbe und fett
                let hervor = !kopf && r.first().is_some_and(|c| c.starts_with('▶'));
                for (i, c) in r.into_iter().enumerate() {
                    let code = c.starts_with('$') || c.starts_with('`') || c.chars().next().is_some_and(|x| x.is_ascii_digit() || x == '%' || x == '-');
                    let inhalt = c.replace('`', "");
                    let el: Element<'static, Msg> = if kopf { text(inhalt.to_uppercase()).size(10.5).color(pal.mute).into() } else if hervor { text(inhalt).size(12.5).font(if code { vc_gui_ui::fonts::MONO_FETT } else { vc_gui_ui::fonts::UI_FETT }).color(pal.accent).into() } else if code { text(inhalt).size(12.5).font(vc_gui_ui::fonts::MONO).color(pal.lab).into() } else { text(inhalt).size(12.5).color(pal.text).into() };
                    zeile = zeile.push(container(el).width(breiten.get(i).copied().unwrap_or(60.0)).padding([3, 8]));
                }
                g = g.push(container(zeile).width(Length::Fixed(gesamt)).style(move |_| container::Style { background: if kopf { None } else if zi % 2 == 0 { Some(Color { a: 0.5, ..pal.editor }.into()) } else { None }, border: Border { color: if kopf { pal.line } else { Color::TRANSPARENT }, width: 0.0, radius: 3.0.into() }, ..Default::default() }));
                if kopf { g = g.push(container(space::horizontal()).width(Length::Fixed(gesamt)).height(1).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() })) }
            }
            karte = karte.push(container(g).width(Length::Fixed(gesamt)).padding([2, 0]));
            erste = false; continue
        }
        let kopf = erste || t.starts_with('#'); let t = t.trim_start_matches('#').trim();
        if let Some(b) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) { karte = karte.push(row![text("•").size(12.5).color(pal.accent), iced::widget::rich_text(md_spannen(pal, b, 12.5, pal.text, false))].spacing(6)) }
        else { karte = karte.push(iced::widget::rich_text(md_spannen(pal, t, if kopf { 14.5 } else { 12.5 }, if kopf { pal.text } else { pal.mute }, kopf))) }
        erste = false;
    }
    container(karte).max_width(520).padding([12, 14]).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 8.0.into() }, shadow: iced::Shadow { color: Color { a: 0.35, ..Color::BLACK }, offset: iced::Vector::new(0.0, 3.0), blur_radius: 10.0 }, ..Default::default() }).into()
}
fn akt(b: Bereich) -> Aktionen<Msg> { Aktionen { minimieren: Msg::BereichAkt(b, BA::Min), maximieren: Msg::BereichAkt(b, BA::Max), wiederherstellen: Msg::BereichAkt(b, BA::Wieder) } }
const KUERZEL_MAX: &str = "Cmd+Umschalt+Enter";

fn flaeche(bg: Color, line: Color) -> container::Style { container::Style { background: Some(bg.into()), border: Border { color: line, width: 0.5, radius: 0.0.into() }, ..Default::default() } }
fn chip<'a>(pal: Pal, s: &str) -> Element<'a, Msg> { container(text(s.to_string()).size(11.5).color(pal.accent)).padding([3, 10]).style(move |_| container::Style { border: Border { color: pal.accent, width: 1.0, radius: 12.0.into() }, ..Default::default() }).into() }
fn knopf<'a>(pal: Pal, s: &str, auf: Option<Msg>, c: Color, haupt: bool) -> Element<'a, Msg> {
    let mut b = button(text(s.to_string()).size(13).color(if haupt { if pal.dark { Color::BLACK } else { Color::WHITE } } else { c })).padding([6, 12]).style(move |_, _| button::Style { background: Some(if haupt { pal.ok.into() } else { pal.panel.into() }), border: Border { color: pal.line, width: 1.0, radius: 7.0.into() }, text_color: c, ..Default::default() });
    if let Some(m) = auf { b = b.on_press(m) } b.into()
}
fn reiter_knopf<'a>(pal: Pal, s: &str, sel: bool, m: Msg) -> Element<'a, Msg> { button(text(s.to_string()).size(12.5).color(pal.text)).padding([4, 10]).on_press(m).style(move |_, _| button::Style { background: sel.then(|| pal.panel2.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() }

/// Fenstereinstellungen. Unter Linux ist die Anwendungskennung des Fensters (Wayland `app_id`, X11 `WM_CLASS`) `vintagecoder`: sie muss zu
/// `StartupWMClass` der .desktop-Datei der Pakete passen, sonst zeigt die Leiste (Dock) kein Programmsymbol und fasst die Fenster nicht zusammen.
fn fenster_einstellungen() -> iced::window::Settings {
    #[cfg_attr(not(target_os = "linux"), allow(unused_mut))]
    let mut s = iced::window::Settings { size: fenstergroesse(std::env::var("VC_WIN").ok().as_deref()), icon: iced::window::icon::from_file_data(ICON, None).ok(), ..Default::default() };
    #[cfg(target_os = "linux")]
    { s.platform_specific.application_id = ANWENDUNGSKENNUNG.into(); }
    s
}

/// Anfangsgröße des Fensters: Umgebungsvariable `VC_WIN=BreitexHöhe` (z. B. 1400x860, 480 bis 10000 Punkte), sonst 1600×1000.
/// Unter Wayland entscheidet der Fenstermanager über die Anfangsgröße (GNOME öffnet große Fenster maximiert); X11 hält sich an die Angabe.
fn fenstergroesse(wert: Option<&str>) -> iced::Size {
    let standard = iced::Size::new(1600.0, 1000.0);
    let Some((b, h)) = wert.and_then(|w| w.trim().split_once(['x', 'X'])) else { return standard };
    match (b.trim().parse::<f32>(), h.trim().parse::<f32>()) {
        (Ok(b), Ok(h)) if (480.0..=10000.0).contains(&b) && (320.0..=10000.0).contains(&h) => iced::Size::new(b, h),
        _ => standard,
    }
}

/// Wie `Icon=` und `StartupWMClass=` in `installer/linux/vintagecoder.desktop`
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
const ANWENDUNGSKENNUNG: &str = "vintagecoder";

fn main() -> iced::Result {
    iced::application(App::neu, App::update, App::view)
        .title("VintageCoder")
        .subscription(App::abo)
        .theme(|a: &App| if a.dunkel() { Theme::Dark } else { Theme::Light })
        .scale_factor(|a: &App| a.einst.zoom)
        .exit_on_close_request(false)
        .window(fenster_einstellungen())
        .font(vc_gui_ui::fonts::INTER_REGULAR).font(vc_gui_ui::fonts::INTER_SEMIBOLD).font(vc_gui_ui::fonts::JETBRAINS_REGULAR).font(vc_gui_ui::fonts::JETBRAINS_BOLD)
        .default_font(vc_gui_ui::fonts::UI)
        .antialiasing(false)
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::{Key, Modifiers};
    #[test] fn kuerzel_werden_erkannt() {
        assert!(trifft("Cmd+Umschalt+F", &Key::Character("f".into()), Modifiers::COMMAND | Modifiers::SHIFT));
        assert!(!trifft("Cmd+F", &Key::Character("f".into()), Modifiers::COMMAND | Modifiers::SHIFT));
        assert!(trifft("F7", &Key::Named(keyboard::key::Named::F7), Modifiers::empty())); assert!(trifft("Cmd+Plus", &Key::Character("=".into()), Modifiers::COMMAND));
        assert!(!trifft("Cmd+K", &Key::Character("k".into()), Modifiers::empty()));
    }
    #[test] fn jeder_befehl_hat_eine_ausfuehrung() {
        let (mut app, _) = App::neu(); let tmp = std::env::temp_dir().join(format!("vcgui-befehle-{}", std::process::id())); std::fs::create_dir_all(&tmp).unwrap(); std::fs::write(tmp.join("a.asm"), "start:\n  nop\n").unwrap();
        app.root = tmp.clone(); app.proj = project::laden(&tmp); app.namen = project::dateien(&app.proj); app.tabs.clear(); app.editoren.clear(); einstellungen::test_konfig(tmp.join("konf"));
        for i in befehle::BEFEHLE { let _ = app.befehl(i.cmd); } // darf nicht paniken, auch ohne geöffnete Datei
        app.oeffne("a.asm"); for i in befehle::BEFEHLE { if !matches!(i.cmd, Cmd::Bauen | Cmd::Starten | Cmd::Debuggen | Cmd::DateiOeffnen | Cmd::ProjektOeffnen) { let _ = app.befehl(i.cmd); } }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn dateibaum_markiert_aktive_geoeffnete_und_ungespeicherte_dateien_ordner_mit_geoeffneten_und_zustaende_folgen_dem_modell() {
        let (mut app, tmp0) = studio_app(); let root = std::env::temp_dir().join(format!("vcgui-baum-mark-{}", std::process::id())); let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::write(root.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"targets":[{"id":"c64","system":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["main.asm"]}],"main":"main.asm"}"#).unwrap();
        for f in ["main.asm", "lib/a.inc", "lib/b.inc"] { std::fs::write(root.join(f), "nop\n").unwrap() }
        app.root = root.clone(); app.projekt_neu_laden();
        assert_eq!(app.offene_dateien().len(), app.tabs.iter().filter(|t| matches!(t, Tab::Datei(_))).count());
        let _ = app.update(Msg::OeffneDatei("main.asm".into())); let _ = app.update(Msg::OeffneDatei("lib/a.inc".into()));
        let o = app.offene_dateien(); assert!(o.contains("main.asm") && o.contains("lib/a.inc") && !o.contains("lib/b.inc"), "{o:?}"); assert_eq!(app.name().as_deref(), Some("lib/a.inc"), "aktuelle Datei");
        // zugeklappter Ordner mit geöffneter Datei: Hinweis am Ordner; ★ am Startprogramm
        let _ = app.update(Msg::BaumKlapp("lib".into())); let k = knoten_sammeln(&mut app, 1280, 800); assert!(!k.iter().any(|x| x.text == "b.inc"), "Ordner zu: Kinder weg"); assert!(k.iter().any(|x| x.text == "★"), "Startprogramm trägt den Stern");
        let _ = app.update(Msg::BaumKlapp("lib".into()));
        // ungespeichert: ● an der Datei
        app.editoren.get_mut("lib/a.inc").unwrap().dirty = true; let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|x| x.text == "●"), "ungespeicherte Datei trägt den Punkt");
        app.editoren.get_mut("lib/a.inc").unwrap().dirty = false;
        // Wechsel und Schließen: die Markierung folgt (Modell, nicht Inhalt)
        let _ = app.update(Msg::OeffneDatei("main.asm".into())); assert_eq!(app.name().as_deref(), Some("main.asm"));
        let i = app.tabs.iter().position(|t| matches!(t, Tab::Datei(n) if n == "lib/a.inc")).unwrap(); let _ = app.update(Msg::Schliesse(i)); assert!(!app.offene_dateien().contains("lib/a.inc"), "geschlossen = nicht mehr markiert");
        // Farbtabelle: jede Art hat eine Rollenfarbe, Marken bleiben Text
        let p = Pal::dark(); use vc_ide_model::project::Group; assert_eq!(p.art_farbe(Group::Asm), p.accent); assert_ne!(p.art_farbe(Group::Asm), p.art_farbe(Group::Basic)); assert_eq!(Group::Asm.tag(), "ASM");
        let _ = std::fs::remove_dir_all(&root); let _ = std::fs::remove_dir_all(&tmp0);
    }
    #[test] fn art_farben_des_dateibaums_haben_in_allen_paletten_genug_kontrast_auf_jeder_flaeche() {
        use vc_ide_model::project::Group;
        let mut bad = vec![];
        for (n, p, grenze) in [("dunkel", Pal::dark(), 4.5f32), ("hell", Pal::light(), 4.5), ("kontrast", Pal::hoch(), 7.0)] {
            for g in Group::ALL { let c = p.art_farbe(g); for (fn_, f) in [("panel", p.panel), ("panel2", p.panel2), ("sel", p.sel)] { let k = kontrast(c, f); if k < grenze { bad.push(format!("{n}: {:?} auf {fn_} {k:.1}:1 (< {grenze})", g)) } } }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }
    #[test] fn dateibaum_zeigt_alle_dateien_klappt_merkt_filtert_und_oeffnet_bilder_im_standardprogramm() {
        let (mut app, tmp0) = studio_app(); let root = std::env::temp_dir().join(format!("vcgui-baum-app-{}", std::process::id())); let _ = std::fs::remove_dir_all(&root);
        for d in ["lib", "gfx/sub", "leer"] { std::fs::create_dir_all(root.join(d)).unwrap() }
        std::fs::write(root.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"targets":[{"id":"c64","system":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["main.asm"]}]}"#).unwrap();
        for f in ["main.asm", "notiz.txt", "daten.bin", "projektbild.png", "lib/music.inc", "gfx/sub/x.txt"] { std::fs::write(root.join(f), if f.ends_with(".bin") { vec![0u8, 1, 2] } else if f.ends_with(".png") { vec![137u8, 80, 78, 71, 0] } else { b"x".to_vec() }).unwrap() }
        let datei = root.join("tree-test.json"); app.baum_datei = Some(datei.clone()); app.root = root.clone(); app.projekt_neu_laden();
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let t = texte(&mut app); for n in ["gfx/", "lib/", "leer/", "main.asm", "notiz.txt", "daten.bin", "projektbild.png", "music.inc", "x.txt"] { assert!(t.iter().any(|x| x == n), "{n} im Baum: {t:?}") }
        // Ordner zuklappen: Kinder weg, Zustand gemerkt (je Projekt) und beim Neuladen wieder da
        let _ = app.update(Msg::BaumKlapp("lib".into())); let t = texte(&mut app); assert!(t.iter().any(|x| x == "lib/") && !t.iter().any(|x| x == "music.inc"), "zugeklappt: {t:?}");
        let gem = vc_ide_model::treestate::load_from(&datei, &root); assert!(gem.contains("lib"), "gemerkt: {gem:?}");
        app.baum_zu.clear(); app.baum_zu = vc_ide_model::treestate::load_from(&datei, &root); assert!(!texte(&mut app).iter().any(|x| x == "music.inc"), "nach Neuladen weiter zu");
        let _ = app.update(Msg::BaumKlapp("lib".into())); assert!(texte(&mut app).iter().any(|x| x == "music.inc"));
        // Filter
        let _ = app.update(Msg::BaumFilter("MUS".into())); let t = texte(&mut app); assert!(t.iter().any(|x| x == "music.inc") && !t.iter().any(|x| x == "main.asm"), "{t:?}"); let _ = app.update(Msg::BaumFilter(String::new()));
        // Bild und Binärdatei: nicht als Text, sondern im Standardprogramm (Attrappe)
        let _ = aussen::aufgezeichnet(); let _ = app.update(Msg::OeffneDatei("projektbild.png".into())); let _ = app.update(Msg::OeffneDatei("daten.bin".into()));
        let a = aussen::aufgezeichnet(); assert!(a.iter().any(|x| matches!(x, aussen::Aufruf::Standardprogramm(p) if p.ends_with("projektbild.png"))) && !a.iter().any(|x| matches!(x, aussen::Aufruf::Standardprogramm(p) if p.ends_with("daten.bin"))), "Bild beim System, Binärdatei in der Hex-Ansicht: {a:?}");
        assert!(app.hexe.contains_key("daten.bin") && app.name().as_deref() == Some("daten.bin"), "Binärdatei öffnet in der Hex-Ansicht"); let t = texte(&mut app); assert!(t.iter().any(|x| x.starts_with("00000000  ")), "Hex-Zeilen sichtbar: {:?}", &t[..t.len().min(12)]);
        let _ = aussen::aufgezeichnet(); let _ = app.update(Msg::HexAkt("daten.bin".into(), hexansicht::HAkt::Extern)); assert!(aussen::aufgezeichnet().iter().any(|x| matches!(x, aussen::Aufruf::Standardprogramm(p) if p.ends_with("daten.bin"))), "Hex-Ansicht kann extern öffnen");
        app.ziel = Some("notiz.txt".into()); let _ = app.befehl(Cmd::HexAnsicht); assert!(app.hexe.contains_key("notiz.txt"), "Befehl „Als Hex ansehen“ geht auch bei Textdateien");
        assert!(!app.editoren.contains_key("projektbild.png") && !app.editoren.contains_key("daten.bin"), "kein Texteditor für Bild/Binärdatei");
        // Textdatei öffnet weiter im Editor; „Neue Datei“ bei einer Datei legt im Ordner der Datei an
        let _ = app.update(Msg::OeffneDatei("notiz.txt".into())); assert!(app.editoren.contains_key("notiz.txt"));
        app.ziel = Some("lib/music.inc".into()); let _ = app.befehl(Cmd::NeueDatei); assert!(matches!(&app.baum.as_ref().expect("Dialog").art, BaumArt::Neu { ordner, .. } if ordner == "lib"), "Ordner der gewählten Datei");
        // Ordnerwahl wie im Speichern-Dialog des Systems: Auswahl ändert den Zielordner, Vorschau zeigt Pfad mit ergänzter Endung
        let _ = app.update(Msg::BaumOrdnerWahl("gfx".into())); let _ = app.update(Msg::BaumText("neu".into())); let t = texte(&mut app); assert!(t.iter().any(|x| x == "Wird angelegt: gfx/neu.asm"), "Vorschau: {t:?}"); assert!(t.iter().any(|x| x == "Ordner"));
        let _ = app.update(Msg::BaumOk); assert!(root.join("gfx/neu.asm").is_file() && app.baum.is_none(), "legt im gewählten Ordner an");
        let _ = std::fs::remove_dir_all(&root); let _ = std::fs::remove_dir_all(&tmp0);
    }
    #[test] fn abgelegte_dateien_kommen_in_den_gewaehlten_baumordner_sonst_in_den_projektordner() {
        let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone()));
        let ext = std::env::temp_dir().join(format!("vcgui-ablage-baum-{}", std::process::id())); let _ = std::fs::remove_dir_all(&ext); std::fs::create_dir_all(&ext).unwrap(); std::fs::write(ext.join("musik.sid"), b"x").unwrap(); std::fs::write(ext.join("bild.png"), b"y").unwrap();
        // nichts gewählt: Projektordner, keine Regel nach Dateiart
        let _ = app.update(Msg::Abgelegt(ext.join("bild.png"))); assert_eq!(app.import.as_ref().expect("Rückfrage").zeilen[0].plan.dir, "", "Projektordner"); let _ = app.update(Msg::ImportAbbruch);
        // Ordner im Baum angeklickt: dort hinein; Ordner im Dialog änderbar
        std::fs::create_dir_all(tmp.join("assets/audio")).unwrap(); app.projekt_neu_laden(); let _ = app.update(Msg::BaumKlapp("assets/audio".into()));
        let _ = app.update(Msg::Abgelegt(ext.join("musik.sid"))); assert_eq!(app.import.as_ref().unwrap().zeilen[0].plan.dir, "assets/audio");
        let _ = app.update(Msg::ImportOrdner(0, "/sound/".into())); assert_eq!(app.import.as_ref().unwrap().zeilen[0].plan.dir, "sound", "Ordner tippbar");
        let _ = app.update(Msg::ImportOk); assert!(tmp.join("sound/musik.sid").is_file(), "übernommen nach sound/");
        // Datei im Baum geöffnet: ihr Ordner ist das Ziel
        let _ = app.update(Msg::OeffneDatei("sprite_demo.asm".into())); assert_eq!(app.baum_ziel, "");
        let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&ext);
    }
    #[test] fn keine_vorgegebene_projektstruktur_mehr_in_der_oberflaeche() {
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden();
        let alle = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let _ = app.befehl(Cmd::ProjektEinstellungen); let t = alle(&mut app); assert!(!t.iter().any(|x| x == "Projektstruktur" || x.starts_with("Wohin kommen abgelegte")), "Reiter Projekt ohne Projektstruktur: {t:?}");
        let _ = app.befehl(Cmd::ProjektNeu); let t = alle(&mut app); assert!(!t.iter().any(|x| x == "Projektstruktur"), "Neues Projekt ohne Strukturwahl: {t:?}"); assert!(t.iter().any(|x| x == "Dateistruktur"), "die Vorschau bleibt");
        assert!(!vc_gui_ui::befehle::BEFEHLE.iter().any(|b| b.id == "dateien_ordnen" || b.id == "neuordnung_zurueck"), "Befehle zum Neuordnen sind aus der Tabelle");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn baum_legt_an_benennt_um_und_loescht() {
        let (mut app, _) = App::neu(); let tmp = std::env::temp_dir().join(format!("vcgui-baum-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        app.root = tmp.clone(); app.papierkorb = false; app.projekt_neu_laden(); app.tabs.clear(); app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        let tippe = |app: &mut App, m: Msg, x: &str| { let _ = app.update(m); let _ = app.update(Msg::BaumText(x.into())); let _ = app.update(Msg::BaumOk); };
        tippe(&mut app, Msg::BaumNeu(String::new(), NeuArt::Asm), "lib/musik.asm");
        assert!(tmp.join("lib/musik.asm").is_file() && app.namen.contains(&"lib/musik.asm".to_string()) && app.name().as_deref() == Some("lib/musik.asm"));
        tippe(&mut app, Msg::BaumNeu(String::new(), NeuArt::Asm), "lib/musik.asm"); assert!(app.baum.as_ref().is_some_and(|b| !b.fehler.is_empty()), "vorhandene Datei bleibt im Dialog mit Fehler"); let _ = app.update(Msg::BaumAbbruch);
        let _ = app.update(Msg::BaumUmbenennen); assert_eq!(app.baum.as_ref().map(|b| b.text.clone()).as_deref(), Some("musik.asm"), "der Dialog zeigt nur den Dateinamen"); let _ = app.update(Msg::BaumText("klang/ton.asm".into())); let _ = app.update(Msg::BaumOk);
        assert!(app.baum.as_ref().is_some_and(|b| !b.fehler.is_empty()) && tmp.join("lib/musik.asm").is_file(), "ein Pfad im Namen wird abgelehnt (Verschieben ist Ziehen)"); let _ = app.update(Msg::BaumAbbruch);
        tippe(&mut app, Msg::BaumUmbenennen, "ton.asm");
        assert!(tmp.join("lib/ton.asm").is_file() && !tmp.join("lib/musik.asm").exists() && app.name().as_deref() == Some("lib/ton.asm") && app.editoren.contains_key("lib/ton.asm") && !app.editoren.contains_key("lib/musik.asm"));
        // Ordner umbenennen: Inhalt, offene Editoren und Tabs ziehen mit
        app.ziel = Some("lib".into()); tippe(&mut app, Msg::BaumUmbenennen, "klang");
        assert!(tmp.join("klang/ton.asm").is_file() && !tmp.join("lib").exists() && app.name().as_deref() == Some("klang/ton.asm") && app.editoren.contains_key("klang/ton.asm"), "Ordner-Umbenennen führt Editoren und Tabs nach");
        let _ = app.update(Msg::BaumLoeschen); assert!(matches!(app.baum.as_ref().map(|b| &b.art), Some(BaumArt::Loeschen(_)))); let _ = app.update(Msg::BaumOk);
        assert!(!tmp.join("klang/ton.asm").exists() && app.tabs.is_empty() && !app.namen.iter().any(|n| n.contains("ton")));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn baum_ziehen_verschiebt_zieht_include_pfade_nach_und_laesst_sich_zuruecknehmen() {
        let (mut app, _) = App::neu(); let tmp = std::env::temp_dir().join(format!("vcgui-zug-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(tmp.join("lib")).unwrap(); std::fs::create_dir_all(tmp.join("src")).unwrap();
        std::fs::write(tmp.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"targets":[{"id":"c64","system":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["main.asm"]}],"main":"main.asm"}"#).unwrap();
        std::fs::write(tmp.join("main.asm"), "* = $0801\n  .include \"lib/a.inc\"\n  .incbin \"lib/d.bin\"\n  rts\n").unwrap(); std::fs::write(tmp.join("lib/a.inc"), "x = 1\n").unwrap(); std::fs::write(tmp.join("lib/d.bin"), [1u8, 2, 3]).unwrap();
        app.papierkorb = false; assert!(app.projekt_oeffnen(tmp.clone())); app.tabs.clear(); app.editoren.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("main.asm"); assert!(app.editoren.contains_key("main.asm"));
        // Klick ohne Bewegung (Drücken und Loslassen auf derselben Zeile) öffnet wie bisher
        let _ = app.update(Msg::BaumZugStart("lib/a.inc".into(), false)); let _ = app.update(Msg::BaumZugLos("lib/a.inc".into())); assert!(app.editoren.contains_key("lib/a.inc") && app.baum_zug.is_none(), "Klick öffnet die Datei");
        // Ziehen auf einen Ordner verschiebt, die Pfade im Quelltext ziehen mit, offene Editoren werden neu gelesen
        let _ = app.update(Msg::BaumZugStart("lib/a.inc".into(), false)); let _ = app.update(Msg::BaumZugUeber("src".into())); assert_eq!(app.baum_zug.as_ref().and_then(|g| g.ziel.clone()).as_deref(), Some("src")); let _ = app.update(Msg::BaumZugLos("src".into()));
        assert!(tmp.join("src/a.inc").is_file() && !tmp.join("lib/a.inc").exists(), "Datei liegt im Zielordner: {:?}", app.log.last());
        assert!(std::fs::read_to_string(tmp.join("main.asm")).unwrap().contains(".include \"src/a.inc\""), "Include-Pfad angepasst");
        assert!(app.editoren["main.asm"].text().contains("src/a.inc") && app.editoren.contains_key("src/a.inc") && !app.editoren.contains_key("lib/a.inc"), "Editoren folgen");
        // Ordner auf Ordner ziehen
        let _ = app.update(Msg::BaumZugStart("lib".into(), true)); let _ = app.update(Msg::BaumZugUeber("src".into())); let _ = app.update(Msg::BaumZugLos("src".into()));
        assert!(tmp.join("src/lib/d.bin").is_file() && !tmp.join("lib").exists() && std::fs::read_to_string(tmp.join("main.asm")).unwrap().contains("src/lib/d.bin"), "Ordner samt Inhalt verschoben: {:?}", app.log.last());
        // in sich selbst: abgelehnt
        let _ = app.update(Msg::BaumZugStart("src".into(), true)); let _ = app.update(Msg::BaumZugUeber("src/lib".into())); let _ = app.update(Msg::BaumZugLos("src/lib".into())); assert!(tmp.join("src/lib/d.bin").is_file());
        // Rückgängig: zuerst Ordner, dann Datei
        let _ = app.befehl(Cmd::BaumRueckgaengig); assert!(tmp.join("lib/d.bin").is_file() && std::fs::read_to_string(tmp.join("main.asm")).unwrap().contains("\"src/a.inc\""), "erster Schritt zurück: {:?}", app.log.last());
        let _ = app.befehl(Cmd::BaumRueckgaengig); assert!(tmp.join("lib/a.inc").is_file() && std::fs::read_to_string(tmp.join("main.asm")).unwrap().contains(".include \"lib/a.inc\""), "zweiter Schritt zurück: {:?}", app.log.last());
        assert!(app.editoren["main.asm"].text().contains("\"lib/a.inc\""), "Editor wieder aktuell");
        // Mit der Maus: Drücken auf der Datei, Ziehen auf die Ordnerzeile, Loslassen
        { use iced::{mouse, Event, Point}; std::fs::create_dir_all(tmp.join("src")).unwrap(); app.baum_zu.clear(); let _ = app.update(Msg::SeiteWahl(Seite::Dateien)); let _ = app.update(Msg::BaumNeuLaden);
          let (qx, qy) = ort(&mut app, "d.bin"); let (zx, zy) = ort(&mut app, "src/"); let (q, z) = (Point::new(qx, qy), Point::new(zx, zy));
          let msgs = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: q }), q), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), q), (Event::Mouse(mouse::Event::CursorMoved { position: z }), z), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), z)]);
          for m in msgs { let _ = app.update(m); }
          assert!(tmp.join("src/d.bin").is_file(), "Maus-Ziehen verschiebt: {:?}", app.log.last()); }
        // Loslassen außerhalb der Zeilen bricht ab
        let _ = app.update(Msg::BaumZugStart("lib/d.bin".into(), false)); let _ = app.update(Msg::BaumZugAbbruch); assert!(app.baum_zug.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn hilfe_und_zeichen_im_panel() {
        let (mut app, _) = App::neu(); let tmp = std::env::temp_dir().join(format!("vcgui-panel-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.asm"), ".text \"hallo\"\n  nop\n").unwrap();
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.editoren.clear(); einstellungen::test_konfig(tmp.join("konf"));
        let _ = app.befehl(Cmd::Hilfe); assert!(app.einst.panel && app.panel_seite == PanelSeite::Hilfe && !app.hilfe.themen.is_empty());
        let _ = app.update(Msg::HilfeSuche("lda".into())); assert!(!app.hilfe.treffer.is_empty());
        let id = app.hilfe.treffer[0].id.clone(); let _ = app.update(Msg::HilfeOeffne(id)); assert!(app.hilfe.doc.is_some());
        let _ = app.update(Msg::HilfeSuche("lda".into())); assert!(app.hilfe.doc.is_none(), "neue Suche zeigt wieder die Treffer");
        app.oeffne("a.asm"); let ed = app.editoren.get_mut("a.asm").unwrap(); ed.cursor = Pos { line: 0, col: 9 };
        let list = app.zeichen_eintraege(vc_ide_model::lang::Lang::Asm); let i = list.iter().position(|e| e.name == "clr").expect("{clr} in der Liste");
        let f0 = app.ed_fokus; let _ = app.update(Msg::ZeichenEinfuegen(ZeichenWahl::Steuer(i)));
        assert!(app.editoren["a.asm"].lines[0].contains("{clr}"), "{}", app.editoren["a.asm"].lines[0]); assert_eq!(app.ed_fokus, f0 + 1, "Editor bekommt die Tastatur zurück");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn editor_teilen_ziehen_und_schliessen() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); let tmp = std::env::temp_dir().join(format!("vcgui-teilen-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for f in ["a.asm", "b.asm", "c.asm"] { std::fs::write(tmp.join(f), "nop\n").unwrap() }
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.asm"); let _ = app.befehl(Cmd::Teilen); assert!(app.andere.is_none() && app.log.last().is_some_and(|l| l == t::TEILEN_MIND_ZWEI), "mit einem Reiter lässt sich nicht teilen");
        for f in ["b.asm", "c.asm"] { app.oeffne(f) }
        let _ = app.befehl(Cmd::Teilen);
        assert_eq!(app.tabs, [Tab::Datei("c.asm".into())]); assert!(app.fokus_rechts); assert_eq!(app.andere.as_ref().unwrap().0.len(), 2, "c rechts und fokussiert, a und b links");
        assert_eq!(app.name().as_deref(), Some("c.asm"));
        // Reiter von links nach rechts ziehen: Druck auf Reiter 0 der linken Gruppe, Bewegung, Loslassen über dem rechten Bereich
        let _ = app.update(Msg::TabDruck(false, 0)); assert_eq!(app.name().as_deref(), Some("a.asm"), "Druck fokussiert die Gruppe und wählt den Reiter");
        let _ = app.update(Msg::Maus(iced::Point::new(300.0, 80.0))); let _ = app.update(Msg::Maus(iced::Point::new(1000.0, 300.0))); let _ = app.update(Msg::MausLos);
        assert!(app.fokus_rechts && app.tabs.contains(&Tab::Datei("a.asm".into())) && app.tabs.len() == 2, "a liegt jetzt rechts: {:?}", app.tabs);
        // ohne Bewegung ist es ein Klick (kein Verschieben)
        let _ = app.update(Msg::TabDruck(true, 0)); let _ = app.update(Msg::MausLos); assert_eq!(app.tabs.len(), 2);
        // Klick im anderen Editor holt dessen Gruppe nach vorn
        let _ = app.update(Msg::EdIn(false, EdMsg::SelectAll)); assert!(!app.fokus_rechts);
        // letzten Reiter links schließen: rechte Gruppe wird die einzige
        let links = app.tabs.len(); for _ in 0..links { let _ = app.update(Msg::SchliesseIn(false, 0)); }
        assert!(app.andere.is_none() && app.tabs.len() == 2 && !app.fokus_rechts);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn streifen_klappt_das_panel_ein_und_aus_und_merkt_den_reiter() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); let tmp = std::env::temp_dir().join(format!("vcgui-streifen-{}", std::process::id())); einstellungen::test_konfig(&tmp);
        assert!(!app.einst.panel, "Vorgabe: eingeklappt");
        let _ = app.update(Msg::StreifenKlick(PanelSeite::Zeichen)); assert!(app.einst.panel && app.panel_seite == PanelSeite::Zeichen && app.einst.panel_reiter == 2);
        let _ = app.update(Msg::StreifenKlick(PanelSeite::Hilfe)); assert!(app.einst.panel && app.panel_seite == PanelSeite::Hilfe, "anderes Symbol wechselt den Reiter");
        let _ = app.update(Msg::StreifenKlick(PanelSeite::Hilfe)); assert!(!app.einst.panel, "erneuter Klick klappt ein");
        let _ = app.befehl(Cmd::Panel); assert!(app.einst.panel && app.panel_seite == PanelSeite::Hilfe, "Cmd+Alt+B öffnet mit dem gemerkten Reiter"); let _ = app.befehl(Cmd::Panel); assert!(!app.einst.panel);
        assert_eq!(app.streifen_zustand().iter().map(|x| x.1).collect::<Vec<_>>(), [false, false, false, false]);
        app.fenster = iced::Size::new(800.0, 600.0); assert!(app.schmal()); app.fenster = iced::Size::new(1280.0, 800.0); assert!(!app.schmal());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn bereiche_minimieren_maximieren_wiederherstellen_und_merken() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); let tmp = std::env::temp_dir().join(format!("vcgui-layout-{}", std::process::id())); einstellungen::test_konfig(&tmp);
        assert_eq!(app.layout.anzeige(Bereich::Schublade), Anzeige::Normal);
        let _ = app.update(Msg::BereichAkt(Bereich::Schublade, BA::Min)); assert_eq!(app.layout.anzeige(Bereich::Schublade), Anzeige::Minimiert); assert!(!app.einst.schublade, "gemerkt");
        let _ = app.update(Msg::Schublade(1)); assert_eq!(app.layout.anzeige(Bereich::Schublade), Anzeige::Normal, "Klick auf einen Reiter klappt auf");
        let _ = app.update(Msg::BereichAkt(Bereich::Schublade, BA::Max)); assert_eq!(app.layout.maximiert(), Some(Bereich::Schublade)); assert_eq!(app.layout.anzeige(Bereich::Dokument), Anzeige::Minimiert);
        assert!(app.einst.seitenleiste && app.einst.schublade, "Maximiert wird nicht gemerkt");
        let _ = app.update(Msg::BereichAkt(Bereich::Schublade, BA::Wieder)); assert!(app.layout.maximiert().is_none() && app.layout.offen(Bereich::Seitenleiste));
        // Kürzel-Befehle: Fokus entscheidet, welcher Bereich maximiert wird
        let _ = app.update(Msg::StreifenKlick(PanelSeite::Hilfe)); assert_eq!(app.fokus_bereich, Bereich::Panel);
        let _ = app.befehl(Cmd::MaximiereFokus); assert_eq!(app.layout.maximiert(), Some(Bereich::Panel));
        let _ = app.befehl(Cmd::MaximiereFokus); assert!(app.layout.maximiert().is_none());
        let _ = app.befehl(Cmd::DokumentMax); assert_eq!(app.layout.maximiert(), Some(Bereich::Dokument)); let _ = app.befehl(Cmd::LayoutZuruecksetzen); assert_eq!(app.layout, Layout::vorgabe());
        // Cmd+B und Cmd+J klappen ihren Bereich ein und wieder auf
        let _ = app.befehl(Cmd::Seitenleiste); assert_eq!(app.layout.anzeige(Bereich::Seitenleiste), Anzeige::Minimiert); let _ = app.update(Msg::SpalteAuf(Seite::Suche)); assert!(app.layout.offen(Bereich::Seitenleiste) && app.seite == Seite::Suche);
        // Zeichnen aller Zustände darf nicht paniken, Bildschirmleser bekommt Namen mit Zustand
        for b in Bereich::ALLE { app.layout.maximieren(b); let _ = app.view(); let _ = app.bereiche_zustand(); } app.layout.wiederherstellen();
        let z = app.bereiche_zustand(); assert_eq!(z[1].2.len(), 1, "Dokumentenfeld hat nur Maximieren"); assert!(z[2].2.contains(&"Schublade minimieren".to_string()) && z[2].2.contains(&"Schublade maximieren".to_string()));
        assert!(trifft("Cmd+Umschalt+Enter", &Key::Named(keyboard::key::Named::Enter), Modifiers::COMMAND | Modifiers::SHIFT));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn reiter_lassen_sich_per_befehl_verschieben() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); let tmp = std::env::temp_dir().join(format!("vcgui-reiter-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for f in ["a.asm", "b.asm", "c.asm"] { std::fs::write(tmp.join(f), "nop\n").unwrap() }
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        for f in ["a.asm", "b.asm", "c.asm"] { app.oeffne(f) }
        let namen = |a: &App| a.tabs.iter().map(|t| match t { Tab::Datei(n) => n.clone(), _ => String::new() }).collect::<Vec<_>>();
        let _ = app.befehl(Cmd::ReiterLinks); assert_eq!(namen(&app), ["a.asm", "c.asm", "b.asm"]); assert_eq!(app.name().as_deref(), Some("c.asm"));
        let _ = app.befehl(Cmd::ReiterLinks); assert_eq!(namen(&app), ["c.asm", "a.asm", "b.asm"]);
        let _ = app.befehl(Cmd::ReiterLinks); assert_eq!(namen(&app), ["c.asm", "a.asm", "b.asm"], "am Anfang bleibt es");
        let _ = app.befehl(Cmd::ReiterRechts); assert_eq!(namen(&app), ["a.asm", "c.asm", "b.asm"]); let _ = app.befehl(Cmd::ReiterRechts); assert_eq!(namen(&app), ["a.asm", "b.asm", "c.asm"]);
        let f0 = app.ed_fokus; let _ = app.update(Msg::BaumNeu(String::new(), NeuArt::Asm)); let _ = app.update(Msg::BaumText("neu.asm".into())); let _ = app.update(Msg::BaumOk); assert_eq!(app.ed_fokus, f0 + 1, "nach dem Anlegen bekommt der Editor die Tastatur");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Echte Oberflächenereignisse gegen die gebaute Ansicht laufen lassen (ohne Fenster, mit dem Software-Renderer)
    thread_local! { static CACHE: std::cell::RefCell<Option<iced_runtime::user_interface::Cache>> = const { std::cell::RefCell::new(None) }; }
    fn schriften_laden() { static EINMAL: std::sync::Once = std::sync::Once::new(); EINMAL.call_once(|| { let mut fs = iced_graphics::text::font_system().write().unwrap(); for b in vc_gui_ui::fonts::alle() { fs.load_font(std::borrow::Cow::Borrowed(b)) } }) }
    /// Mitte des ersten Textknotens mit genau diesem Text (statt fester Koordinaten, die bei jeder Layoutänderung wandern)
    fn ort(app: &mut App, t: &str) -> (f32, f32) {
        let (b, h) = ((app.fenster.width * app.einst.zoom) as u32, (app.fenster.height * app.einst.zoom) as u32);
        let (vorn, t) = t.strip_suffix('*').map_or((false, t), |p| (true, p)); let k = knoten_sammeln(app, b, h); let n = k.iter().find(|k| k.art == "text" && (if vorn { k.text.starts_with(t) } else { k.text == t })).unwrap_or_else(|| panic!("Text „{t}“ nicht gefunden"));
        (n.grenzen.x + n.grenzen.width / 2.0, n.grenzen.y + n.grenzen.height / 2.0)
    }
    /// Obere linke Ecke des größten scrollbaren Bereichs mit mindestens dieser Breite (Raster des Trackers)
    fn scroll_ecke(app: &mut App, min_breite: f32) -> (f32, f32) {
        let (b, h) = ((app.fenster.width * app.einst.zoom) as u32, (app.fenster.height * app.einst.zoom) as u32);
        let k = knoten_sammeln(app, b, h); let n = k.iter().filter(|k| k.art == "scroll" && k.grenzen.width >= min_breite && k.grenzen.y > 100.0).max_by(|a, b| (a.grenzen.width * a.grenzen.height).total_cmp(&(b.grenzen.width * b.grenzen.height))).expect("Scrollbereich");
        (n.grenzen.x, n.grenzen.y)
    }
    fn simuliere(app: &mut App, schritte: &[(iced::Event, iced::Point)]) -> Vec<Msg> {
        use iced_runtime::user_interface::{Cache, UserInterface};
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new({ schriften_laden(); vc_gui_ui::fonts::UI }, iced::Pixels(16.0)));
        let mut cache = CACHE.with(|c| c.borrow_mut().take()).unwrap_or_default(); let mut alle = vec![];
        let stil = iced::advanced::renderer::Style { text_color: iced::Color::WHITE };
        for (n, (e, p)) in schritte.iter().enumerate() {
            let grenze = iced::Size::new(app.fenster.width / app.einst.zoom, app.fenster.height / app.einst.zoom); let mut msgs: Vec<Msg> = vec![];
            { let view = app.view(); let mut ui = UserInterface::build(view, grenze, std::mem::take(&mut cache), &mut renderer);
              if n == 0 { ui.draw(&mut renderer, &iced::Theme::Dark, &stil, iced::mouse::Cursor::Available(*p)); } // erste Zeichnung: Zeichenmaße der Widgets
              let _ = ui.update(&[e.clone()], iced::mouse::Cursor::Available(*p), &mut renderer, &mut iced::advanced::clipboard::Null, &mut msgs);
              ui.draw(&mut renderer, &iced::Theme::Dark, &stil, iced::mouse::Cursor::Available(*p)); cache = ui.into_cache(); }
            for m in msgs { alle.push(m.clone()); let _ = app.update(m); }
        }
        CACHE.with(|c| *c.borrow_mut() = Some(cache));
        alle
    }
    #[test] fn reiter_ziehen_mit_echten_mausereignissen() {
        use iced::{mouse, Event, Point};
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); let tmp = std::env::temp_dir().join(format!("vcgui-zieh-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for f in ["a.asm", "b.asm", "c.asm"] { std::fs::write(tmp.join(f), "nop\n").unwrap() }
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        for f in ["a.asm", "b.asm", "c.asm"] { app.oeffne(f) }
        let namen = |a: &App| a.tabs.iter().map(|t| match t { Tab::Datei(n) => n.clone(), _ => String::new() }).collect::<Vec<_>>();
        assert_eq!(namen(&app), ["a.asm", "b.asm", "c.asm"]);
        let p = |x: f32, y: f32| Point::new(x, y);
        // Tab „a.asm“ (ganz links in der Reiterleiste) greifen und hinter „c.asm“ ziehen
        let sch = [(Event::Mouse(mouse::Event::CursorMoved { position: p(300.0, 72.0) }), p(300.0, 72.0)),
            (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p(300.0, 72.0)),
            (Event::Mouse(mouse::Event::CursorMoved { position: p(340.0, 72.0) }), p(340.0, 72.0)),
            (Event::Mouse(mouse::Event::CursorMoved { position: p(480.0, 72.0) }), p(480.0, 72.0)),
            (Event::Mouse(mouse::Event::CursorMoved { position: p(560.0, 72.0) }), p(560.0, 72.0)),
            (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p(560.0, 72.0))];
        let msgs = simuliere(&mut app, &sch);
        eprintln!("Nachrichten: {msgs:?}\nReiter: {:?}", namen(&app));
        assert!(msgs.iter().any(|m| matches!(m, Msg::TabDruck(..))), "Druck auf den Reiter kommt an");
        assert_eq!(namen(&app).last().map(String::as_str), Some("a.asm"), "a.asm steht jetzt hinten: {:?}", namen(&app));
        // Reiter an den rechten Rand des Editors ziehen: der Editor teilt sich
        let p0 = p(300.0, 72.0); let sch = [(Event::Mouse(mouse::Event::CursorMoved { position: p0 }), p0), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p0),
            (Event::Mouse(mouse::Event::CursorMoved { position: p(500.0, 300.0) }), p(500.0, 300.0)), (Event::Mouse(mouse::Event::CursorMoved { position: p(1100.0, 300.0) }), p(1100.0, 300.0)), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p(1100.0, 300.0))];
        let _ = simuliere(&mut app, &sch); assert!(app.andere.is_some(), "Editor ist geteilt"); assert_eq!(app.tabs.len() + app.andere.as_ref().unwrap().0.len(), 3);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn reiterleiste_folgt_dem_aktiven_reiter_und_cursor_wird_gemerkt() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); let tmp = std::env::temp_dir().join(format!("vcgui-leiste-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for i in 0..30 { std::fs::write(tmp.join(format!("datei{i:02}.asm")), "nop\nlda #1\nsta $d020\n").unwrap() }
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        let f0 = app.ed_fokus; for i in 0..30 { app.oeffne(&format!("datei{i:02}.asm")) }
        assert_eq!(app.ed_fokus, f0 + 30, "jede geöffnete Datei bekommt die Tastatur");
        let (lo, hi) = app.sichtbare_reiter(&app.tabs.clone(), app.aktiv); assert!(hi == 30 && lo > 0 && lo < 29, "der neueste Reiter ist sichtbar, ältere wandern links hinaus: {lo}..{hi}");
        app.aktiv = 0; let (lo, hi) = app.sichtbare_reiter(&app.tabs.clone(), 0); assert!(lo == 0 && hi < 30, "{lo}..{hi}");
        // Cursor merken und beim erneuten Öffnen wiederherstellen
        let ed = app.editoren.get_mut("datei03.asm").unwrap(); ed.set_cursor(Pos { line: 2, col: 4 }, false);
        app.marken_sichern(); assert_eq!(app.marken.get("datei03.asm"), Some(&(3, 5)));
        app.editoren.remove("datei03.asm"); app.tabs.retain(|t| *t != Tab::Datei("datei03.asm".into())); app.aktiv = 0;
        app.oeffne("datei03.asm"); let ed = &app.editoren["datei03.asm"]; assert_eq!((ed.cursor.line, ed.cursor.col), (2, 4), "Cursor steht wieder an der alten Stelle");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn kontextmenues_per_rechtsklick_tastatur_und_befehle() {
        use iced::{keyboard::{key::Named, Key, Modifiers}, mouse, Event, Point};
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); let tmp = std::env::temp_dir().join(format!("vcgui-ktx-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for f in ["a.asm", "b.asm"] { std::fs::write(tmp.join(f), "start:\n  lda #1\n  jmp start\n").unwrap() }
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.asm"); app.oeffne("b.asm");
        let rechts = |app: &mut App, x: f32, y: f32| { let p = Point::new(x, y); simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)), p)]) };
        let taste = |app: &mut App, n: Named| { let _ = app.update(Msg::Taste(Key::Named(n), Modifiers::empty())); };
        let kontext_art = |app: &App| app.kontext.as_ref().map(|k| k.art);
        // Reiter: Rechtsklick auf „a.asm“ wählt ihn und öffnet das Reitermenü
        rechts(&mut app, 300.0, 72.0); assert_eq!(kontext_art(&app), Some(Art::Reiter), "Reitermenü"); assert_eq!(app.name().as_deref(), Some("a.asm"));
        let (z, _) = app.kontext_zeilen(app.kontext.as_ref().unwrap()); let namen: Vec<_> = z.iter().filter(|x| !x.name.is_empty()).map(|x| (x.name.as_str(), x.aktiv)).collect();
        assert!(namen.contains(&("Datei schließen", true)) && namen.contains(&("Reiter nach links verschieben", false)) && namen.contains(&("Reiter nach rechts verschieben", true)), "{namen:?}");
        taste(&mut app, Named::Escape); assert!(app.kontext.is_none(), "Esc schließt");
        // Reiter nach rechts verschieben über das Menü (Pfeil nach unten bis zum Eintrag, Enter)
        rechts(&mut app, 300.0, 72.0); loop { let k = app.kontext.as_ref().unwrap(); let (_, c) = app.kontext_zeilen(k); if c.get(k.sel).copied().flatten() == Some(Cmd::ReiterRechts) { break } taste(&mut app, Named::ArrowDown) }
        taste(&mut app, Named::Enter); assert!(app.kontext.is_none() && app.tabs == [Tab::Datei("b.asm".into()), Tab::Datei("a.asm".into())], "{:?}", app.tabs);
        // Editor: Rechtsklick setzt den Cursor, Haltepunkt über das Menü
        rechts(&mut app, 600.0, 190.0); assert_eq!(kontext_art(&app), Some(Art::Editor), "Editormenü"); let zeile = app.ed().unwrap().cursor.line;
        loop { let k = app.kontext.as_ref().unwrap(); let (_, c) = app.kontext_zeilen(k); if c.get(k.sel).copied().flatten() == Some(Cmd::HaltepunktUmschalten) { break } taste(&mut app, Named::ArrowDown) }
        taste(&mut app, Named::Enter); assert!(app.ed().unwrap().breakpoints.contains(&zeile), "Haltepunkt gesetzt");
        // Dateibaum: Rechtsklick auf eine Datei, „Löschen …“ zielt auf diese Datei (nicht auf die aktive)
        let mut gefunden = false; for y in (90..400).step_by(6) { rechts(&mut app, 100.0, y as f32); if let Some(k) = &app.kontext { if k.art == Art::Baum && k.ziel.as_deref() == Some("a.asm") { gefunden = true; break } } app.kontext = None }
        assert!(gefunden, "Rechtsklick auf a.asm im Baum");
        let _ = app.update(Msg::KontextWahl(Cmd::DateiLoeschen)); assert!(matches!(app.baum.as_ref().map(|b| &b.art), Some(BaumArt::Loeschen(r)) if r == "a.asm"));
        let _ = app.view(); let _ = app.bereiche_zustand();
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Zufällige Bedienung (feste Startzahl): Klicks, Rechtsklicks, Ziehen, Rad, Tasten gegen das echte Beispielprojekt; darf nie paniken
    #[test] fn zufaellige_bedienung_paniert_nicht() {
        use iced::{keyboard::{key::Named, Key, Modifiers}, mouse, Event, Point};
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tmp = std::env::temp_dir().join(format!("vcgui-fuzz-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for e in std::fs::read_dir(repo.join("examples/c64/prallball")).unwrap().flatten() { if e.path().is_file() && !e.file_name().to_string_lossy().starts_with("neu-") { let _ = std::fs::copy(e.path(), tmp.join(e.file_name())); } }
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf")); app.papierkorb = false;
        let namen = app.namen.clone(); for n in namen.iter().take(6) { app.oeffne(n) }
        let mut z: u64 = std::env::var("VC_FUZZ_SEED").ok().and_then(|x| x.parse().ok()).unwrap_or(0x9E3779B97F4A7C15); let mut zufall = move |m: u64| { z ^= z << 13; z ^= z >> 7; z ^= z << 17; z % m };
        let tasten = [Named::ArrowDown, Named::ArrowUp, Named::ArrowLeft, Named::ArrowRight, Named::Enter, Named::Escape, Named::Backspace, Named::Delete, Named::Tab, Named::Home, Named::End, Named::PageUp, Named::PageDown, Named::F1, Named::F9, Named::F12];
        for runde in 0..1500 {
            let p = Point::new(zufall(1280) as f32, zufall(800) as f32);
            let mut sch: Vec<(Event, Point)> = vec![(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)];
            if zufall(6) == 0 { let _ = app.update(Msg::Fenster(iced::Size::new(200.0 + zufall(1400) as f32, 100.0 + zufall(900) as f32))); }
            if zufall(8) == 0 { let b = Bereich::ALLE[zufall(4) as usize]; let a = [BA::Min, BA::Max, BA::Wieder][zufall(3) as usize]; let _ = app.update(Msg::BereichAkt(b, a)); }
            if zufall(8) == 0 { let c = befehle::BEFEHLE[zufall(befehle::BEFEHLE.len() as u64) as usize].cmd; if !matches!(c, Cmd::Bauen | Cmd::Starten | Cmd::Debuggen | Cmd::DateiOeffnen | Cmd::ProjektOeffnen | Cmd::ImOrdnerZeigen | Cmd::PfadKopieren | Cmd::Einfuegen | Cmd::Kopieren | Cmd::Ausschneiden | Cmd::Einstellungen) { let _ = app.befehl(c); } }
            if zufall(10) == 0 { let _ = app.update(Msg::Ed(EdMsg::Resize(1 + zufall(120) as usize, 1 + zufall(60) as usize))); }
            if zufall(12) == 0 { let _ = app.update(Msg::SetZoom(0.5 + zufall(30) as f32 / 10.0)); }
            match zufall(9) {
                0 | 1 => { sch.push((Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p)); sch.push((Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)) }
                2 => { sch.push((Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)), p)) }
                3 => { let q = Point::new(zufall(1280) as f32, zufall(800) as f32); sch.push((Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p)); sch.push((Event::Mouse(mouse::Event::CursorMoved { position: q }), q)); sch.push((Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), q)) }
                4 => { sch.push((Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Lines { x: zufall(7) as f32 - 3.0, y: zufall(9) as f32 - 4.0 } }), p)) }
                5 => { sch.push((Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p)); sch.push((Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p)); sch.push((Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)) }
                _ => { let k = tasten[zufall(tasten.len() as u64) as usize]; let m = if zufall(3) == 0 { Modifiers::COMMAND } else { Modifiers::empty() }; let _ = app.update(Msg::Taste(Key::Named(k), m)); let t = ["a", " ", "{", "\"", "$"][zufall(5) as usize]; let _ = app.update(Msg::Ed(EdMsg::Insert(t.into()))); }
            }
            let ergebnis = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _ = simuliere(&mut app, &sch); }));
            if ergebnis.is_err() { panic!("Absturz in Runde {runde}: Ereignisse {sch:?}") }
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Bildschirmfoto der Ansicht (Software-Renderer) als PNG; hilft beim Gestalten, ohne das Fenster zu öffnen
    fn foto(app: &mut App, ereignisse: &[(iced::Event, iced::Point)], pfad: &str) { foto_gross(app, ereignisse, pfad, 1280, 800) }
    /// Ansicht der App in Fenstergröße `b`×`h` (logische Pixel) als PNG
    fn foto_gross(app: &mut App, ereignisse: &[(iced::Event, iced::Point)], pfad: &str, b: u32, h: u32) {
        use iced_runtime::user_interface::{Cache, UserInterface};
        app.fenster = iced::Size::new(b as f32 / app.einst.zoom, h as f32 / app.einst.zoom);
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new({ schriften_laden(); vc_gui_ui::fonts::UI }, iced::Pixels(16.0)));
        let mut cache = Cache::default(); let mut cursor = iced::mouse::Cursor::Unavailable;
        let theme = if app.dunkel() { iced::Theme::Dark } else { iced::Theme::Light };
        for (e, p) in ereignisse.iter().chain(std::iter::once(&(iced::Event::Window(iced::window::Event::RedrawRequested(std::time::Instant::now())), ereignisse.last().map_or(iced::Point::ORIGIN, |x| x.1)))) {
            cursor = iced::mouse::Cursor::Available(*p); let mut msgs: Vec<Msg> = vec![];
            let last = matches!(e, iced::Event::Window(_));
            { let view = app.view(); let mut ui = UserInterface::build(view, iced::Size::new(b as f32, h as f32), std::mem::take(&mut cache), &mut renderer);
              let _ = ui.update(&[e.clone()], cursor, &mut renderer, &mut iced::advanced::clipboard::Null, &mut msgs);
              if last { ui.draw(&mut renderer, &theme, &iced::advanced::renderer::Style { text_color: iced::Color::WHITE }, cursor) }
              cache = ui.into_cache(); }
            for m in msgs { let _ = app.update(m); }
        }
        let _ = cursor;
        if let iced::Renderer::Secondary(r) = &mut renderer {
            let mut pix = tiny_skia::Pixmap::new(b, h).unwrap(); let mut maske = tiny_skia::Mask::new(b, h).unwrap();
            let vp = iced_graphics::Viewport::with_physical_size(iced::Size::new(b, h), 1.0);
            r.draw(&mut pix.as_mut(), &mut maske, &vp, &[iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(b as f32, h as f32))], iced::Color::BLACK);
            for px in pix.data_mut().chunks_exact_mut(4) { px.swap(0, 2) } pix.save_png(pfad).unwrap();
        }
    }
    /// Ein einzelnes Element als PNG (für Gestaltung ohne Fenster)
    fn foto_element(el: Element<'_, Msg>, b: u32, h: u32, hintergrund: iced::Color, pfad: &str) {
        use iced_runtime::user_interface::{Cache, UserInterface};
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new({ schriften_laden(); vc_gui_ui::fonts::UI }, iced::Pixels(16.0)));
        let wurzel: Element<'_, Msg> = container(el).padding(20).width(Length::Fill).height(Length::Fill).into();
        let mut ui = UserInterface::build(wurzel, iced::Size::new(b as f32, h as f32), Cache::default(), &mut renderer);
        ui.draw(&mut renderer, &iced::Theme::Dark, &iced::advanced::renderer::Style { text_color: iced::Color::WHITE }, iced::mouse::Cursor::Unavailable);
        if let iced::Renderer::Secondary(r) = &mut renderer {
            let mut pix = tiny_skia::Pixmap::new(b, h).unwrap(); let mut maske = tiny_skia::Mask::new(b, h).unwrap();
            let vp = iced_graphics::Viewport::with_physical_size(iced::Size::new(b, h), 1.0);
            r.draw(&mut pix.as_mut(), &mut maske, &vp, &[iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(b as f32, h as f32))], hintergrund);
            for px in pix.data_mut().chunks_exact_mut(4) { px.swap(0, 2) } pix.save_png(pfad).unwrap();
        }
    }
    /// Eine App mit Quelldatei, Bauinfo und vorgetäuschter Debug-Sitzung (Zustandsabbild ohne Emulator)
    fn debug_app() -> (App, std::path::PathBuf) {
        static ZAEHLER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0); let tmp = std::env::temp_dir().join(format!("vcgui-dbg-{}-{}", std::process::id(), ZAEHLER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.asm"), ".target c64\n.org $0801\nstart: lda #$20\n  sta $d020\n  jmp start\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.asm");
        app.info = Some(BauInfo { target: "c64".into(), artifact: tmp.join("build/a.prg"), symbols: [("start".to_string(), 0x0801i64)].into(), run: Some(0x0801), segmente: vec![(0x0801, 9)], zeilen: vec![("a.asm".into(), 3, 0x0801, 2), ("a.asm".into(), 4, 0x0803, 3), ("a.asm".into(), 5, 0x0806, 3)] });
        let dl = |addr: u16, b: &[u8], t: &str| debug::DisLine { addr, bytes: b.to_vec(), text: t.into(), illegal: false, cycles: 2, extra: false };
        let snap = debug::Snap { have_regs: true, running: false, pc: 0x0803, a: 0x20, x: 1, y: 2, sp: 0xFD, flags: 0x24, disasm: vec![dl(0x0801, &[0xA9, 0x20], "lda #$20"), dl(0x0803, &[0x8D, 0x20, 0xD0], "sta $d020"), dl(0x0806, &[0x4C, 1, 8], "jmp $0801")], mem_addr: 0x0400, mem: (0..=255u8).collect(), watch: vec![("$d020".into(), Ok(14))], detected: Some(Ok(vc_ide::snapshot::Detected { system: "c64".into(), supported: true, mode: "Text".into(), screen: 0x0400, rom_off: Some(0), ..Default::default() })), points: vec![debug::PointInfo { id: 1, kind: "exec".into(), start: 0x0803, end: 0x0803, hits: 2, ..Default::default() }], ..Default::default() };
        app.dbg = Some(debug::Debugger::demo(snap)); app.beob = vec!["$d020".into()];
        (app, tmp)
    }
    #[test] fn debug_sitzung_markiert_die_zeile_und_nimmt_befehle_an() {
        let (mut app, tmp) = debug_app();
        assert!(app.nutzbar(Cmd::DebugWeiter) && app.nutzbar(Cmd::DebugEnde));
        let _ = app.update(Msg::DebugTick);
        assert_eq!(app.editoren["a.asm"].debug_line, Some(3), "PC $0803 steht in Zeile 4 (0-basiert 3)"); assert_eq!(app.snap.pc, 0x0803);
        assert_eq!(app.info.as_ref().unwrap().adresse_von("a.asm", 5), Some(0x0806)); assert_eq!(app.info.as_ref().unwrap().marke_bei(0x0801), Some("start"));
        // Haltepunkt im Editor (Zeile 5) wird als Adresse gewünscht
        app.editoren.get_mut("a.asm").unwrap().breakpoints.insert(4); assert_eq!(app.gewuenschte_haltepunkte(), [0x0806u16].into_iter().collect());
        // alle Teile des Panels lassen sich zeichnen (Register, Disassembler, Speicher, Beobachten, Haltepunkte)
        app.panel_seite = PanelSeite::Debug; app.layout.aufklappen(Bereich::Panel);
        let _ = simuliere(&mut app, &[]);
        for m in [Msg::SnapErkennen, Msg::SnapName("ball".into()), Msg::SnapSprites(false), Msg::SnapKacheln(true), Msg::SnapRom(false), Msg::SnapSpeichern] { let _ = app.update(m); } let _ = simuliere(&mut app, &[]);
        let _ = app.update(Msg::BeobText("zaehler".into())); let _ = app.update(Msg::BeobAdd); assert_eq!(app.beob.len(), 2); let _ = app.update(Msg::BeobDel(1)); assert_eq!(app.beob.len(), 1);
        // „Bis hierher ausführen“ auf einer Codezeile merkt den einmaligen Haltepunkt, auf einer Zeile ohne Code nicht
        app.editoren.get_mut("a.asm").unwrap().set_cursor(Pos { line: 3, col: 0 }, false); let _ = app.befehl(Cmd::DebugHierher); assert_eq!(app.run_to, Some((0x0803, false)));
        app.run_to = None; app.editoren.get_mut("a.asm").unwrap().set_cursor(Pos { line: 0, col: 0 }, false); let _ = app.befehl(Cmd::DebugHierher); assert!(app.run_to.is_none() && app.log.last().unwrap().contains("keinen Code"));
        let _ = app.befehl(Cmd::DebugEnde); assert!(app.dbg.is_none() && !app.nutzbar(Cmd::DebugWeiter), "Ende beendet die Sitzung");
        assert!(app.editoren["a.asm"].debug_line.is_none() && app.panel_seite != PanelSeite::Debug);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn speicherkarte_zeigt_die_belegung_des_programms_und_nimmt_wahl_an() {
        let (mut app, tmp) = debug_app(); app.dbg = None;
        let _ = app.update(Msg::PanelWahl(PanelSeite::Karte)); assert_eq!(app.karte.sel, 8, "erste belegte Seite ($0801 → $08)");
        assert!(app.karte.banking.is_some(), "C64 hat Bank-Presets"); assert_eq!(app.karte.facts.as_ref().unwrap().segs.len(), 1);
        let _ = app.update(Msg::KarteFilter("prog")); let _ = app.update(Msg::KarteSeite(0xD0)); let _ = app.update(Msg::KartePreset("all_ram".into()));
        app.layout.aufklappen(Bereich::Panel); let _ = simuliere(&mut app, &[]);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_karte() { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.update(Msg::PanelWahl(PanelSeite::Karte)); app.layout.aufklappen(Bereich::Panel); foto(&mut app, &[], "/tmp/vc-karte.png"); let _ = std::fs::remove_dir_all(&tmp); }
    /// Ist-Aufnahmen des Debug-Bereichs (Vorschlag zur Überarbeitung): `cargo test -p vc-gui bild_debug_ist -- --ignored`
    #[test] #[ignore] fn bild_debug_ist() {
        let ziel = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../build/debug-ist"); std::fs::create_dir_all(&ziel).unwrap();
        for (schema, name) in [(Schema::Dunkel, "dunkel"), (Schema::Hell, "hell")] {
            let zustaende: [(&str, Box<dyn Fn(&mut App, &mut debug::Snap)>); 8] = [
                ("angehalten", Box::new(|_, _| {})),
                ("laufend", Box::new(|_, s| { s.running = true; s.have_regs = false; s.disasm.clear(); s.mem.clear(); })),
                ("startet", Box::new(|_, s| { s.starting = true; s.have_regs = false; s.disasm.clear(); s.mem.clear(); })),
                ("fehler", Box::new(|a, s| { s.step_block = Some("Einzelschritt ist mit dieser VICE-Version gesperrt.".into()); s.watch = vec![("d020".into(), Err("„d020“ ist weder Register noch Symbol. Meinst du $d020? Beispiele: $d020 (Byte an dieser Adresse), a, x, y, sp, pc, start (Symbol), [$d020] + 1.".into()))]; a.beob = vec!["d020".into()]; a.log.push("Die Verbindung zum Emulator ist verloren (er wurde beendet oder ist abgestürzt). Die Debug-Sitzung wird beendet.".into()); })),
                ("momentaufnahme", Box::new(|_, s| { s.mem.clear(); })),
                ("karte", Box::new(|a, _| { a.panel_seite = PanelSeite::Karte; a.karte_laden(); })),
                ("bild", Box::new(|a, s| { s.can_frame = true; s.frame_sources = vec!["VIC-II".into(), "VDC".into()]; let (w, h) = (384usize, 272usize); let rgba: Vec<u8> = (0..w * h).flat_map(|i| { let (x, y) = (i % w, i / w); let rand = x < 32 || x >= 352 || y < 36 || y >= 236; if rand { [0x88, 0x7e, 0xcb, 255] } else { [0x40, 0x31, 0x8d, 255] } }).collect(); s.frame = Some(std::sync::Arc::new(vc_emu::streams::video::Frame { width: w, height: h, rgba, frame_no: 1 })); s.frame_seq = 1; s.can_text = true; a.panel_seite = PanelSeite::Emulator; })),
                ("tests", Box::new(|a, _| { a.schublade = 2; a.panel_seite = PanelSeite::Gliederung; })),
            ];
            for (zn, f) in zustaende.iter() {
                let (mut app, tmp) = debug_app(); app.einst.schema = schema.clone();
                let mut snap = app.dbg.as_ref().unwrap().snapshot(); f(&mut app, &mut snap); app.dbg = Some(debug::Debugger::demo(snap));
                if *zn != "karte" && *zn != "bild" && *zn != "tests" { app.schublade = 3 }
                if *zn == "tests" { app.dbg = None; use vc_ide::tests::suite::{Event, Group, Item}; use vc_ide::tests::{Level, Outcome}; let fl = std::path::PathBuf::from("/tmp/x/prallball.vctest"); app.tests.suite.apply(Event::Static(Group { title: "Statische Prüfungen".into(), items: vec![Item { file: fl.clone(), outcome: Outcome::passed("keine Überlappung", None) }, Item { file: fl.clone(), outcome: Outcome::failed("Programm kleiner als $2000", Level::Error, "erwartet ≤ $2000, ist $2310") }], ..Default::default() })); }
                app.layout.aufklappen(Bereich::Schublade); if matches!(*zn, "karte" | "bild") { app.layout.aufklappen(Bereich::Panel) }
                let _ = app.update(Msg::DebugTick); if *zn == "karte" { app.panel_seite = PanelSeite::Karte } if *zn == "bild" { app.panel_seite = PanelSeite::Emulator }
                foto(&mut app, &[], &ziel.join(format!("{zn}-{name}.png")).to_string_lossy());
                let _ = std::fs::remove_dir_all(&tmp);
            }
        }
    }
    #[test] #[ignore] fn bild_debug_panel() { let (mut app, tmp) = debug_app(); let _ = app.update(Msg::DebugTick); app.panel_seite = PanelSeite::Debug; app.layout.aufklappen(Bereich::Panel); app.editoren.get_mut("a.asm").unwrap().breakpoints.insert(4); foto(&mut app, &[], "/tmp/vc-debug.png"); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] #[ignore] fn bild_hover_modus() {
        let (app, _) = App::neu(); let pal = app.pal();
        let md = "**LDA** – Akkumulator laden\n\nFlags: `N Z`\n\n| Modus | Opcode | Bytes | Zyklen |\n|---|---|---|---|\n| Indirekt, X-vorindiziert (izx) | $A1 | 2 | 6 |\n| Zeropage (zp) | $A5 | 2 | 3 |\n| ▶ Zeropage, X-indiziert (zpx) | $B5 | 2 | 4 |\n| Absolut (abs) | $AD | 3 | 4 |\n\nDiese Zeile: **Zeropage, X-indiziert (zpx)** (vom Assembler gewählt) · Bytes `B5 10` · 4 Zyklen\n";
        foto_element(md_karte(pal, md), 640, 330, pal.editor, "/tmp/vc-hover-modus.png");
    }
    #[test] #[ignore] fn bild_hover_karte() {
        use iced::{mouse, Event, Point};
        let data = vc_ide::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let tmp = std::env::temp_dir().join(format!("vcgui-foto-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.asm"), ".target c64\n.org $0801\nstart: lda #$20\n  sta $d020\n  jmp start\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.asm");
        let (l, rx) = lsp::Lsp::start(&tmp, &data); let u = lsp::uri(&tmp, "a.asm");
        l.senden(lsp::Cmd::Open { uri: u.clone(), text: app.editoren["a.asm"].text() });
        for (req, zeile, spalte) in [(1u64, 3usize, 8usize), (2, 3, 3), (3, 2, 9)] { l.senden(lsp::Cmd::Hover { uri: u.clone(), zeile, spalte, req }) }
        let mut texte: Vec<(u64, Option<String>)> = vec![]; let t0 = std::time::Instant::now();
        while t0.elapsed().as_secs() < 20 && texte.len() < 3 { if let Ok(lsp::Antwort::Hover { req, text, .. }) = rx.recv_timeout(std::time::Duration::from_millis(200)) { texte.push((req, text)) } }
        texte.sort_by_key(|x| x.0);
        for (n, (req, text)) in texte.iter().enumerate() {
            eprintln!("--- Hover {req}: {text:?}");
            let Some(t) = text else { continue };
            app.editoren.get_mut("a.asm").unwrap().hover = Some(vc_gui_editor::core::HoverInfo { von: Pos { line: 0, col: 0 }, bis: Pos { line: 0, col: 1 }, text: t.clone() });
            let pal = app.pal(); foto_element(md_karte(pal, t), 640, 380, pal.editor, &format!("/tmp/vc-hover-{}.png", n + 1));
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_gesamt() {
        let tmp = std::env::temp_dir().join(format!("vcgui-gesamt-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("main.asm"), ".target c64\n.org $0801\nstart: lda #$20\n  sta $d020\n  jmp start\n").unwrap(); std::fs::write(tmp.join("lib.asm"), "nop\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("main.asm"); app.oeffne("lib.asm");
        foto(&mut app, &[], "/tmp/vc-gesamt.png"); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn hover_im_basic_zeigt_register_und_zerlegt_den_wert_ohne_sprachdienst() {
        let tmp = std::env::temp_dir().join(format!("vcgui-bashov-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.bas"), "; Bitmap\npoke 53265, 59\nprint \"53265\"\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.bas"); app.ed().unwrap().set_cursor(Pos { line: 0, col: 0 }, false);
        // Maus über der Adresse: Registerkarte, obwohl der Textcursor woanders steht
        let _ = app.update(Msg::Ed(EdMsg::Hover(Some(Pos { line: 1, col: 7 }))));
        let h = app.ed().unwrap().hover.clone().expect("Karte über der Adresse"); assert!(h.text.contains("vic2.ctrl1") && h.text.contains("BMM") && h.von.line == 1 && h.enthaelt(Pos { line: 1, col: 7 }), "{h:?}");
        // über dem Wert: Zerlegung in Felder
        let _ = app.update(Msg::Ed(EdMsg::Hover(Some(Pos { line: 1, col: 13 }))));
        let h = app.ed().unwrap().hover.clone().expect("Karte über dem Wert"); assert!(h.text.contains("| BMM | 5 | 1 |") && h.text.contains("setzt:"), "{h:?}");
        // Kommentar und Zeichenkette: keine Karte
        let _ = app.update(Msg::Ed(EdMsg::Hover(Some(Pos { line: 0, col: 3 })))); assert!(app.ed().unwrap().hover.is_none());
        let _ = app.update(Msg::Ed(EdMsg::Hover(Some(Pos { line: 2, col: 9 })))); assert!(app.ed().unwrap().hover.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn hover_folgt_dem_mauszeiger_und_nie_dem_textcursor() {
        use iced::{mouse, Event, Point};
        let tmp = std::env::temp_dir().join(format!("vcgui-hov-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.asm"), ".target c64\n.org $0801\nstart: lda #$20\n  sta $d020\n  jmp start\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        let rx = LSP_EMPFAENGER.lock().unwrap().take().expect("Empfänger"); // eigener Sprachdienst für diesen Test
        let (lsp_neu, rx2) = lsp::Lsp::start(&tmp, &app.data); drop(rx); app.lsp = lsp_neu;
        app.oeffne("a.asm"); app.ed().unwrap().set_cursor(Pos { line: 0, col: 3 }, false);
        let cw = 7.8_f32; let (x0, y0, lh) = (330.0_f32, 94.0_f32, 19.0_f32); let ort = |zeile: usize, spalte: f32| Point::new(x0 + (spalte + 0.5) * cw, y0 + zeile as f32 * lh);
        let bewege = |app: &mut App, p: Point| simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]);
        let warte_und_zeichne = |app: &mut App, p: Point| { std::thread::sleep(std::time::Duration::from_millis(520)); simuliere(app, &[(Event::Window(iced::window::Event::RedrawRequested(std::time::Instant::now())), p)]) };
        let antwort = |app: &mut App| { let t0 = std::time::Instant::now(); while t0.elapsed().as_secs() < 20 { if let Ok(a) = rx2.recv_timeout(std::time::Duration::from_millis(200)) { if matches!(a, lsp::Antwort::Hover { .. }) { let _ = app.update(Msg::Lsp(a)); return } else { let _ = app.update(Msg::Lsp(a)); } } } panic!("keine Hover-Antwort (hover_req={}, gesendet={:?})", app.hover_req, app.lsp_gesendet.keys().collect::<Vec<_>>()) };
        // 1. Textcursor steht in Zeile 0, die Maus über „$d020“ (Zeile 3): die Karte gehört zu „$d020“
        let p = ort(3, 8.0); bewege(&mut app, p);
        let m = warte_und_zeichne(&mut app, p); let anfrage = m.iter().find_map(|x| if let Msg::Ed(EdMsg::Hover(Some(z))) | Msg::EdIn(_, EdMsg::Hover(Some(z))) = x { Some(*z) } else { None }).expect("Hover-Anfrage nach der Verzögerung");
        assert_eq!((anfrage.line, anfrage.col >= 6 && anfrage.col <= 10), (3, true), "Zelle unter der Maus, nicht beim Textcursor: {anfrage:?}");
        antwort(&mut app);
        let h = app.ed().unwrap().hover.clone().expect("Karte"); assert!(h.text.contains("D020") && h.text.contains("53280") && h.von.line == 3 && h.enthaelt(anfrage), "{h:?}");
        // 2. Der Textcursor wandert bei ruhender Maus: die Karte bleibt
        let _ = app.update(Msg::Ed(EdMsg::Move { dx: 0, dy: 1, extend: false })); let _ = app.update(Msg::Ed(EdMsg::Move { dx: 3, dy: 0, extend: false }));
        assert_eq!(app.ed().unwrap().hover.as_ref().map(|x| x.text.clone()), Some(h.text.clone()), "Karte bleibt bei Cursorbewegung");
        // 3. Maus bewegt sich innerhalb desselben Wortes: Karte bleibt ohne neue Anfrage
        let n = app.hover_req; bewege(&mut app, ort(3, 9.0)); assert!(app.ed().unwrap().hover.is_some() && app.hover_req == n, "gleiches Wort: Karte bleibt");
        // 4. Maus über ein anderes Wort („sta“): alte Karte weg, nach der Verzögerung die neue
        let q = ort(3, 3.0); bewege(&mut app, q); assert!(app.ed().unwrap().hover.is_none(), "anderes Wort: alte Karte weg");
        let m = warte_und_zeichne(&mut app, q); assert!(m.iter().any(|x| matches!(x, Msg::EdIn(_, EdMsg::Hover(Some(z))) | Msg::Ed(EdMsg::Hover(Some(z))) if z.line == 3 && (2..=4).contains(&z.col))), "{m:?}");
        antwort(&mut app); assert!(app.ed().unwrap().hover.as_ref().is_some_and(|h| h.text.contains("STA")), "{:?}", app.ed().unwrap().hover);
        // 5. Maus verlässt den Textbereich: Karte weg
        bewege(&mut app, Point::new(100.0, 300.0)); assert!(app.ed().unwrap().hover.is_none(), "Maus draußen: Karte weg");
        // 6. Hover wieder aufbauen, Scrollen schließt
        let p = ort(3, 8.0); bewege(&mut app, p); warte_und_zeichne(&mut app, p); antwort(&mut app); assert!(app.ed().unwrap().hover.is_some());
        simuliere(&mut app, &[(Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 } }), p)]); assert!(app.ed().unwrap().hover.is_none(), "Scrollen schließt"); app.ed().unwrap().scroll_row = 0.0;
        // 7. Tippen schließt
        bewege(&mut app, Point::new(100.0, 300.0)); let p = ort(3, 8.0); bewege(&mut app, p); warte_und_zeichne(&mut app, p); antwort(&mut app); assert!(app.ed().unwrap().hover.is_some());
        let _ = app.update(Msg::Ed(EdMsg::Insert("x".into()))); assert!(app.ed().unwrap().hover.is_none(), "Tippen schließt");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// W203 (Sprung automatisch verlängert, mit `.branchlong auto` selbst bestellt) zählt nicht unter „Probleme“, sondern steht als eine Protokollzeile
    #[test] fn lange_spruenge_w203_stehen_im_protokoll_nicht_in_den_problemen() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.log.clear();
        let d = |code: &str, sev: &str| serde_json::json!({ "file": "a.asm", "line": 3, "message": "m", "severity": sev, "code": code });
        let _ = app.update(Msg::BauFertig(Ok(serde_json::json!({ "ok": false, "diagnostics": [d("W203", "warning"), d("W203", "warning"), d("W203", "warning"), d("W200", "warning"), d("E202", "error")] }))));
        assert_eq!(app.probleme.len(), 2, "nur die übrigen Meldungen sind Probleme"); assert!(app.probleme.iter().all(|p| !p.text.is_empty()));
        assert!(app.log.iter().any(|l| l == "3 Sprünge automatisch verlängert (.branchlong auto)"), "{:?}", app.log);
        let _ = app.update(Msg::BauFertig(Ok(serde_json::json!({ "ok": false, "diagnostics": [d("W200", "warning")] }))));
        assert_eq!(app.probleme.len(), 1); assert!(!app.log.iter().rev().take(2).any(|l| l.contains("Sprünge")), "ohne W203 keine Zeile");
    }
    /// Lizenzhinweis L001 (Copyleft im Quelltext): Protokollzeile und Information unter „Probleme“, kein Fehler, Bau bleibt gültig
    #[test] fn lizenzhinweis_l001_ist_eine_information_kein_fehler() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.log.clear();
        let hinweis = "Hinweis (L001): Das Programm enthält Quelltext unter GPL-3.0-only (lib/rmtplayr.asm, Radek Sterba).";
        let _ = app.update(Msg::BauFertig(Ok(serde_json::json!({ "ok": true, "size": 10, "artifact": "build/a.xex", "target": "atari8", "diagnostics": [{ "file": "a.asm", "line": 0, "message": hinweis, "severity": "info", "code": "L001" }] }))));
        assert!(app.log.iter().any(|l| l == hinweis), "{:?}", app.log);
        assert_eq!(app.probleme.len(), 1); assert!(!app.probleme[0].fehler, "Information, kein Fehler");
        assert!(app.log.iter().any(|l| l.starts_with("Fertig")), "der Bau gilt als fertig");
    }
    /// Meldung „Emulator sofort beendet“ (aus dem Hintergrund) landet zeilenweise im Protokoll und öffnet es
    #[test] fn emulator_sofortabbruch_steht_im_protokoll() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.log.clear(); app.einst.schublade = false;
        let _ = app.update(Msg::StartFehler(vec![]));
        assert!(app.log.is_empty() && !app.einst.schublade, "ohne Meldung bleibt alles still");
        let _ = app.update(Msg::StartFehler(vec!["Der Emulator hat sich sofort beendet, Rückgabewert 1. Fehlende ROM-Dateien? Siehe Einstellungen ▸ ROMs.\nAusgabe des Emulators:\n  Couldn't load kernal ROM".into()]));
        assert_eq!(app.log.len(), 3, "{:?}", app.log); assert!(app.log[2].contains("kernal ROM") && app.einst.schublade && app.schublade == 0);
    }
    /// Esc schließt den Einrichtungsassistenten und die Befehlsleiste, auch wenn ein Eingabefeld mit Fokus die Taste selbst verbraucht
    #[test] fn esc_schliesst_assistent_und_befehlsleiste() {
        use iced::event::Status; use iced::keyboard::{Event as KE, Key, Modifiers, Location, key::Named};
        let taste = |k: Named| iced::Event::Keyboard(KE::KeyPressed { key: Key::Named(k), modified_key: Key::Named(k), physical_key: iced::keyboard::key::Physical::Unidentified(iced::keyboard::key::NativeCode::Unidentified), location: Location::Standard, modifiers: Modifiers::empty(), text: None, repeat: false });
        assert!(matches!(fenster_ereignis(taste(Named::Escape), Status::Captured), Some(Msg::TasteP(..))), "Esc kommt auch an, wenn ein Eingabefeld es verbraucht hat");
        assert!(fenster_ereignis(taste(Named::ArrowDown), Status::Captured).is_none(), "andere verbrauchte Tasten bleiben beim Widget");
        assert!(matches!(fenster_ereignis(taste(Named::ArrowDown), Status::Ignored), Some(Msg::TasteP(..))));
        let (mut app, tmp) = studio_app(); app.log.clear();
        let _ = app.befehl(Cmd::Assistent); assert!(app.wiz.is_some());
        let _ = app.update(Msg::Taste(Key::Named(Named::Escape), Modifiers::empty())); assert!(app.wiz.is_none(), "Esc bricht den Assistenten ab");
        let _ = app.befehl(Cmd::Palette); assert!(app.palette.offen);
        let _ = app.update(Msg::Taste(Key::Named(Named::Escape), Modifiers::empty())); assert!(!app.palette.offen, "Esc schließt die Befehlsleiste");
        let _ = std::fs::remove_dir_all(tmp);
    }
    /// Die Anwendungskennung des Fensters (Linux: app_id / WM_CLASS) gleicht `StartupWMClass` und `Icon` der .desktop-Datei der Pakete
    #[test] fn anwendungskennung_gleicht_der_desktop_datei() {
        let d = include_str!("../../../installer/linux/vintagecoder.desktop");
        assert!(d.lines().any(|l| l == format!("StartupWMClass={ANWENDUNGSKENNUNG}")), "{d}");
        assert!(d.lines().any(|l| l == format!("Icon={ANWENDUNGSKENNUNG}")));
        let s = fenster_einstellungen(); assert!(s.icon.is_some(), "Fenstersymbol gesetzt");
        #[cfg(target_os = "linux")] assert_eq!(s.platform_specific.application_id, "vintagecoder");
    }
    #[test] fn vc_win_setzt_die_anfangsgroesse() {
        assert_eq!(fenstergroesse(Some("1400x860")), iced::Size::new(1400.0, 860.0));
        assert_eq!(fenstergroesse(Some(" 1280X800 ")), iced::Size::new(1280.0, 800.0));
        for falsch in [None, Some(""), Some("gross"), Some("100x100"), Some("1400"), Some("1400x"), Some("99999x800")] { assert_eq!(fenstergroesse(falsch), iced::Size::new(1600.0, 1000.0), "{falsch:?}"); }
    }
    #[test] fn starten_baut_zuerst_und_startet_nur_nach_erfolg() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.root = std::env::temp_dir(); app.log.clear(); app.namen.clear(); app.tabs.clear();
        let _ = app.befehl(Cmd::Starten); assert_eq!(app.danach, Danach::Nichts, "ohne Quelldatei wird nichts vorgemerkt");
        app.danach = Danach::Start; app.baut = true;
        let _ = app.update(Msg::BauFertig(Ok(serde_json::json!({ "ok": false, "diagnostics": [{ "file": "a.asm", "line": 2, "message": "kaputt", "severity": "error" }] }))));
        assert!(app.info.is_none() && !app.log.iter().any(|l| l.contains("Starten")), "Fehler beim Bauen: kein Start"); assert_eq!(app.probleme.len(), 1);
        app.danach = Danach::Start; app.baut = true;
        let _ = app.update(Msg::BauFertig(Ok(serde_json::json!({ "ok": true, "target": "c64", "artifact": "build/a.prg", "size": 5, "symbols": [{ "name": "start", "value": 49152 }], "lines": [{ "file": "a.asm", "line": 3, "addr": 49152, "len": 2 }], "run": 49152 }))));
        let i = app.info.as_ref().expect("Bauinfo"); assert_eq!((i.target.as_str(), i.run, i.symbols["start"]), ("c64", Some(49152), 49152)); assert_eq!(i.zeile_bei(49153), Some(("a.asm", 3)));
        assert!(app.log.iter().any(|l| l.contains("Starten")), "{:?}", app.log); assert_eq!(app.danach, Danach::Nichts);
        // Ziel-Wahl: Menü auf/zu, Wahl wird gemerkt und im Chip gezeigt
        let _ = app.update(Msg::ZielMenue); assert!(app.ziel_menue); assert!(app.ziel_eintraege().iter().any(|(id, n)| id == "@emulator" && n.starts_with("Emulator (")));
        let _ = app.update(Msg::ZielWahl("@emulator".into())); assert!(!app.ziel_menue && app.run_on == "@emulator");
        let _ = app.update(Msg::Gestartet(Err("kein Emulator".into()))); assert!(app.log.last().unwrap().contains("Start fehlgeschlagen"));
    }
    #[test] fn test_explorer_laeuft_zeigt_gruppen_zustaende_und_detail() {
        let data = vc_ide::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let tmp = std::env::temp_dir().join(format!("vcgui-tests-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("main.asm"), ".target c64\n.org $c000\nstart: lda #1\n  sta $d020\n  rts\n").unwrap();
        let b = vc_ide::build::build(&vc_ide::build::Ctx { root: &tmp, data: &data }, "main.asm", None).unwrap(); assert_eq!(b["ok"], true, "{b}");
        std::fs::write(tmp.join("main.vctest"), r#"{ "kind": "test", "target": "c64", "program": "build/main.prg", "static": [ { "check": "overlap" }, { "check": "size", "max": 100 }, { "check": "size", "max": 1 } ] }"#).unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.data = data; app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false;
        let _ = app.befehl(Cmd::TestsAusfuehren); assert_eq!(app.schublade, 2); assert!(app.tests.laeuft());
        let t0 = std::time::Instant::now(); while app.tests.laeuft() && t0.elapsed().as_secs() < 30 { std::thread::sleep(std::time::Duration::from_millis(50)); let _ = app.update(Msg::TestTick); }
        let c = app.tests.suite.counts(); assert!(!app.tests.laeuft() && c.passed >= 2 && c.failed == 1, "{c:?} {:?}", app.tests.suite.error);
        let (f, n) = { let g = &app.tests.suite.groups[0]; let i = g.items.iter().find(|i| i.outcome.state == vc_ide::tests::State::Failed).unwrap(); (i.file.clone(), i.outcome.name.clone()) };
        let _ = app.update(Msg::TestWahl(f, n)); let _ = app.update(Msg::TestFilter("size".into())); let _ = app.update(Msg::TestNurFehler(true)); app.layout.aufklappen(Bereich::Schublade);
        let _ = simuliere(&mut app, &[]);
        let _ = app.update(Msg::TestFehler); let t0 = std::time::Instant::now(); while app.tests.laeuft() && t0.elapsed().as_secs() < 30 { std::thread::sleep(std::time::Duration::from_millis(50)); let _ = app.update(Msg::TestTick); }
        assert_eq!(app.tests.suite.counts().failed, 1, "Wiederholung der Fehlgeschlagenen");
        let _ = app.update(Msg::TestDebug(tmp.join("main.vctest"), "x".into())); assert!(app.dbg.is_none(), "im Testlauf startet kein Emulator");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_tests_schublade() {
        use vc_ide::tests::suite::{Event, Group};
        use vc_ide::tests::{Level, Outcome, RegSnap};
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.tabs.clear(); app.editoren.clear(); app.schublade = 2;
        let f = std::path::PathBuf::from("/tmp/x/prallball.vctest");
        app.tests.suite.apply(Event::Static(Group { title: "Statische Prüfungen".into(), items: vec![vc_ide::tests::suite::Item { file: f.clone(), outcome: Outcome::passed("keine Überlappung", None) }, vc_ide::tests::suite::Item { file: f.clone(), outcome: Outcome::failed("Programm kleiner als $2000", Level::Error, "erwartet ≤ $2000, ist $2310") }], ..Default::default() }));
        app.tests.suite.apply(Event::FileStarted { file: f.clone(), title: "prallball.vctest".into(), backend: Some("VICE x64sc".into()), expected: 3, mailbox: false });
        app.tests.suite.apply(Event::Outcome { file: f.clone(), outcome: Outcome::passed("rand_byte liefert 0..39", Some("$1F".into())).with_regs(RegSnap { pc: 0xCF00, a: 0x1F, x: 0, y: 2, sp: 0xFD, flags: 0x24 }) });
        app.tests.suite.apply(Event::Outcome { file: f.clone(), outcome: Outcome::skipped("Joystick links", "keine Hardware") });
        app.tests.suite.apply(Event::Outcome { file: f.clone(), outcome: Outcome::timed_out("Warteschleife endet", "nach 5 s nicht zurückgekehrt") });
        app.tests.suite.apply(Event::FileDone { file: f.clone(), millis: 1840 });
        app.tests.sel = Some((f, "rand_byte liefert 0..39".into()));
        foto(&mut app, &[], "/tmp/vc-tests.png");
    }
    #[test] fn register_aenderungen_und_haltepunkt_rand_im_disassembler() {
        let (mut app, tmp) = debug_app();
        let _ = app.update(Msg::DebugTick); assert!(app.reg_alt.is_none() && app.reg_neu.is_some(), "erster Halt: noch nichts zum Vergleichen");
        // nächster Halt mit anderen Werten: der alte Stand wird „war …“, nur Geändertes zählt
        let mut s2 = app.dbg.as_ref().unwrap().snapshot(); s2.pc = 0x0806; s2.a = 0x21; s2.flags = 0x26; app.dbg = Some(debug::Debugger::demo(s2));
        let _ = app.update(Msg::DebugTick); let (alt, neu) = (app.reg_alt.unwrap(), app.reg_neu.unwrap());
        assert_eq!((alt[0], neu[0], alt[1], neu[1], alt[2] == neu[2]), (0x0803, 0x0806, 0x20, 0x21, true)); assert_eq!(alt[5] ^ neu[5], 0x02, "Flag Z geändert");
        app.panel_seite = PanelSeite::Debug; app.layout.aufklappen(Bereich::Panel); let _ = simuliere(&mut app, &[]);
        // Klick auf den Rand: Haltepunkt in der Quellzeile der Adresse (0x0803 = Zeile 4) umschalten, zweiter Klick nimmt ihn zurück
        let _ = app.update(Msg::DisHaltepunkt(0x0803)); assert!(app.editoren["a.asm"].breakpoints.contains(&3));
        let _ = app.update(Msg::DisHaltepunkt(0x0803)); assert!(app.editoren["a.asm"].breakpoints.is_empty());
        let _ = app.update(Msg::DisHaltepunkt(0x9999)); assert!(app.log.last().unwrap().contains("keine Quellzeile"));
        let _ = app.befehl(Cmd::DebugEnde); assert!(app.reg_alt.is_none() && app.reg_neu.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn im_ordner_zeigen_startet_im_testlauf_nie_einen_prozess_und_wird_aufgezeichnet() {
        use iced::keyboard::{key::Named, Key, Modifiers};
        let tmp = std::env::temp_dir().join(format!("vcgui-ordner-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap(); std::fs::write(tmp.join("a.asm"), "nop\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear();
        assert!(aussen::gesperrt() && !app.papierkorb, "Testlauf: Sicherung an, kein echter Papierkorb"); let _ = aussen::aufgezeichnet();
        app.oeffne("a.asm"); let erwartet = project::pfad(&tmp, "a.asm");
        // Befehl direkt
        let _ = app.befehl(Cmd::ImOrdnerZeigen); assert_eq!(aussen::aufgezeichnet(), vec![aussen::Aufruf::ImOrdnerZeigen(erwartet.clone())]);
        // über das Kontextmenü der Datei (Eintrag wählen und Enter) und über die Palette
        let _ = app.update(Msg::KontextAuf(Art::Baum, iced::Point::new(120.0, 140.0), Some("a.asm".into())));
        loop { let k = app.kontext.as_ref().unwrap(); let (_, c) = app.kontext_zeilen(k); if c.get(k.sel).copied().flatten() == Some(Cmd::ImOrdnerZeigen) { break } let _ = app.update(Msg::Taste(Key::Named(Named::ArrowDown), Modifiers::empty())); }
        let _ = app.update(Msg::Taste(Key::Named(Named::Enter), Modifiers::empty())); assert_eq!(aussen::aufgezeichnet(), vec![aussen::Aufruf::ImOrdnerZeigen(erwartet.clone())], "Kontextmenü");
        let _ = app.update(Msg::KontextWahl(Cmd::ImOrdnerZeigen)); assert_eq!(aussen::aufgezeichnet().len(), 1, "Menüwahl");
        let _ = app.befehl(Cmd::Palette); let _ = app.update(Msg::PaletteFrage("Im Ordner zeigen".into())); let _ = app.update(Msg::Taste(Key::Named(Named::Enter), Modifiers::empty()));
        assert!(aussen::aufgezeichnet().len() <= 1, "Palette: höchstens ein aufgezeichneter Aufruf"); 
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Im GUI-Quelltext darf kein Prozess direkt gestartet werden: alles läuft über `aussen` (dort steht die Sicherung)
    #[test] fn prozesse_werden_im_gui_quelltext_nur_ueber_die_schnittstelle_gestartet() {
        let verboten = [concat!("Command", "::new"), concat!("process", "::Command"), concat!(".spa", "wn()"), concat!("open", "_command"), concat!("xdg", "-open")];
        for e in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap() {
            let p = e.unwrap().path(); if p.extension().is_none_or(|x| x != "rs") || p.file_name().is_some_and(|n| n == "aussen.rs") { continue }
            let t = std::fs::read_to_string(&p).unwrap();
            for v in verboten { assert!(!t.contains(v), "{} enthält „{v}“: Aufrufe nach außen gehören in aussen.rs", p.display()) }
        }
    }
    /// App mit geöffnetem Sprite-Studio (Beispielsprite in einem Temp-Projekt)
    fn beispiel_projekte() -> Vec<std::path::PathBuf> {
        let ex = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples"); let mut v = vec![];
        for sys in std::fs::read_dir(&ex).unwrap().flatten() { if let Ok(rd) = std::fs::read_dir(sys.path()) { for pr in rd.flatten() { if let Ok(d) = std::fs::read_dir(pr.path()) { for f in d.flatten() { if f.path().extension().is_some_and(|e| e == "vcprj") { v.push(f.path()) } } } } } }
        v.sort(); v
    }
    #[test] fn alle_beispielprojekte_laden_mit_baum_und_startprogramm() {
        let alle = beispiel_projekte(); assert!(alle.len() >= 20, "Beispielprojekte gefunden: {}", alle.len());
        for f in alle {
            let z = project::ziel_fuer(&f).expect("Ziel"); assert_eq!(z.vcprj.as_deref(), f.file_name().and_then(|n| n.to_str()), "{}", f.display());
            let p = project::pruefen(&z.root, z.vcprj.clone()).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
            assert!(p.pf.is_some(), "{}: Projektdaten fehlen", f.display());
            if p.pf.as_ref().unwrap().source_files().is_empty() { assert!(project::startprogramm(&p).is_none(), "{}", f.display()); continue } // leeres Projekt (nur Ziel, keine Quellen) ist gültig
            assert!(!p.entries.is_empty(), "{}: Baum leer", f.display());
            let start = project::startprogramm(&p).unwrap_or_else(|| panic!("{}: kein Startprogramm", f.display())); assert!(p.entries.iter().any(|e| e.rel == start), "{}: Start {start} nicht im Baum", f.display());
            assert!(p.pf.as_ref().unwrap().targets.iter().all(|t| !t.id.is_empty() || !t.system.is_empty()), "{}: Ziel ohne Namen", f.display());
        }
    }
    #[test] fn projekt_oeffnen_ordner_projektdatei_und_asm_darin() {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."); let sd = repo.join("examples/c64/sprite_demo").canonicalize().unwrap();
        for weg in [sd.clone(), sd.join("sprite_demo.vcprj"), sd.join("sprite_demo.asm"), sd.join("graphics/hero.vcsprite")] {
            let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); einstellungen::test_konfig(std::env::temp_dir().join(format!("vcgui-konf-open-{}", std::process::id())));
            assert!(app.projekt_oeffnen(weg.clone()), "{}", weg.display());
            assert_eq!(app.root, sd, "{}", weg.display()); assert_eq!(app.proj.file.as_deref(), Some("sprite_demo.vcprj")); assert_eq!(app.proj.name, "C64: sprite_demo");
            assert!(app.proj.pf.is_some() && app.namen.iter().any(|n| n == "graphics/hero.vcsprite") && app.namen.iter().any(|n| n == "sprite_demo.asm"));
            assert!(!app.tabs.is_empty(), "eine Datei ist offen: {}", weg.display());
            let erwartet = if weg.ends_with("hero.vcsprite") { "graphics/hero.vcsprite" } else { "sprite_demo.asm" };
            assert!(matches!(app.tabs.get(app.aktiv), Some(Tab::Datei(n)) if n == erwartet), "{}: {:?}", weg.display(), app.tabs);
            assert!(project::gruppiere(&app.proj.entries).iter().any(|g| g.dateien.iter().any(|d| d.rel == "graphics/hero.vcsprite")));
        }
    }
    #[test] fn projekt_oeffnen_ohne_vcprj_ist_einzeldatei_und_kaputte_projektdatei_meldet_fehler() {
        static Z: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let tmp = std::env::temp_dir().join(format!("vcgui-open-{}-{}", std::process::id(), Z.fetch_add(1, std::sync::atomic::Ordering::SeqCst))); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(tmp.join("lose")).unwrap(); std::fs::create_dir_all(tmp.join("kaputt")).unwrap();
        std::fs::write(tmp.join("lose/x.asm"), ".org $0801
").unwrap(); std::fs::write(tmp.join("kaputt/k.vcprj"), "{ nicht json").unwrap(); std::fs::write(tmp.join("kaputt/k.asm"), "nop\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); einstellungen::test_konfig(std::env::temp_dir().join(format!("vcgui-konf-open-{}", std::process::id())));
        let alt = app.root.clone(); let alt_name = app.proj.name.clone();
        assert!(project::ziel_fuer(&tmp.join("lose/x.asm")).is_none());
        assert!(app.projekt_oeffnen(tmp.join("lose/x.asm"))); assert_eq!(app.root, alt, "Einzeldatei ändert das Projekt nicht"); assert!(app.editoren.contains_key("x.asm") || app.tabs.iter().any(|t| matches!(t, Tab::Datei(n) if n.ends_with("x.asm"))));
        assert!(!app.projekt_oeffnen(tmp.join("kaputt")), "kaputte Projektdatei"); assert_eq!(app.root, alt); assert_eq!(app.proj.name, alt_name);
        assert!(app.log.last().is_some_and(|l| l.starts_with("Projekt konnte nicht geladen werden:") && l.contains("k.vcprj")), "{:?}", app.log.last());
        assert_eq!(app.schublade, 0);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    fn studio_app() -> (App, std::path::PathBuf) {
        static Z: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tmp = std::env::temp_dir().join(format!("vcgui-studio-{}-{}", std::process::id(), Z.fetch_add(1, std::sync::atomic::Ordering::SeqCst))); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::copy(repo.join("examples/c64/sprite_demo/graphics/hero.vcsprite"), tmp.join("hero.vcsprite")).unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("hero.vcsprite"); (app, tmp)
    }
    #[test] fn zeichenflaeche_passt_sich_ein_zoomt_ganzzahlig_mit_cmd_mausrad_und_verschiebt_mit_leertaste_ohne_zu_zeichnen() {
        use iced::{keyboard as K, mouse, Event, Point}; use iced::keyboard::{key, Key, Location, Modifiers};
        let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite";
        let senden = |app: &mut App, ev: &[(Event, Point)]| { let _ = simuliere(app, ev); };
        let p0 = Point::new(730.0, 380.0); senden(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p0 }), p0)]); let p1 = Point::new(731.0, 380.0); senden(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p1 }), p1)]); let _ = simuliere(&mut app, &[]);
        let (pw, ph) = { let s = &app.studios[rel]; s.zoomansicht.platz.get() }; assert!(pw > 300.0 && ph > 200.0, "Zeichenfläche meldet ihre Größe: {pw}×{ph}");
        let fit = { let s = &app.studios[rel]; vc_gui_ui::flaeche::Ansicht::einpass(iced::Size::new(pw, ph), s.doc.aspect * s.doc.width as f32, s.doc.height as f32) }; assert!(fit >= 4.0 && fit == fit.floor(), "Einpassen ist ganzzahlig: {fit}");
        assert!(app.studios[rel].zoomansicht.zoom.is_none(), "Vorgabe: Einpassen");
        let p = Point::new(730.0, 380.0);
        let taste = |k: Key, druck: bool| -> Event { let n = k.clone(); if druck { Event::Keyboard(K::Event::KeyPressed { key: k, modified_key: n, physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified), location: Location::Standard, modifiers: Modifiers::empty(), text: None, repeat: false }) } else { Event::Keyboard(K::Event::KeyReleased { key: k, modified_key: n, physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified), location: Location::Standard, modifiers: Modifiers::empty() }) } };
        // Cmd + Mausrad: eine Stufe größer (ganzzahlig, von „Einpassen“ aus), ohne zu zeichnen
        senden(&mut app, &[(Event::Keyboard(K::Event::ModifiersChanged(Modifiers::COMMAND)), p), (Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 } }), p)]);
        assert_eq!(app.studios[rel].zoomansicht.zoom, Some(fit + 1.0), "Cmd+Rad vergrößert um eine ganze Stufe"); assert!(!app.studios[rel].doc.dirty());
        senden(&mut app, &[(Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 } }), p)]); assert_eq!(app.studios[rel].zoomansicht.zoom, Some(fit), "und wieder kleiner");
        senden(&mut app, &[(Event::Keyboard(K::Event::ModifiersChanged(Modifiers::empty())), p)]);
        // Mausrad ohne Cmd zoomt nicht
        senden(&mut app, &[(Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 } }), p)]); assert_eq!(app.studios[rel].zoomansicht.zoom, Some(fit), "ohne Cmd bleibt das Zoom");
        // passt die Fläche hinein, bleibt sie zentriert: Verschieben ändert nichts
        senden(&mut app, &[(taste(Key::Named(key::Named::Space), true), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::CursorMoved { position: Point::new(790.0, 400.0) }), Point::new(790.0, 400.0)), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), Point::new(790.0, 400.0)), (taste(Key::Named(key::Named::Space), false), p)]);
        assert_eq!(app.studios[rel].zoomansicht.pan, (0.0, 0.0), "eingepasst: nichts zu verschieben"); assert!(!app.studios[rel].doc.dirty(), "Leertaste+Ziehen zeichnet nicht");
        // groß gezoomt: Leertaste + Ziehen verschiebt, begrenzt, ohne Scrollbalken
        let _ = app.update(Msg::StudioAkt(rel.into(), studio::Aktion::Zoom(48.0))); let _ = simuliere(&mut app, &[]);
        senden(&mut app, &[(taste(Key::Named(key::Named::Space), true), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::CursorMoved { position: Point::new(760.0, 395.0) }), Point::new(760.0, 395.0)), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), Point::new(760.0, 395.0)), (taste(Key::Named(key::Named::Space), false), p)]);
        let pan = app.studios[rel].zoomansicht.pan; assert!(pan.0 != 0.0 || pan.1 != 0.0, "verschoben: {pan:?}"); assert!(!app.studios[rel].doc.dirty(), "kein Pixel gezeichnet");
        for _ in 0..30 { let _ = app.update(Msg::StudioAkt(rel.into(), studio::Aktion::Schieben(500.0, 500.0))); } let g = app.studios[rel].zoomansicht.pan; assert!(g.0.abs() < 3000.0 && g.1.abs() < 3000.0, "Verschiebung begrenzt: {g:?}");
        // Einpassen setzt Zoom und Verschiebung zurück
        let _ = app.update(Msg::StudioAkt(rel.into(), studio::Aktion::ZoomAnpassen)); assert!(app.studios[rel].zoomansicht.zoom.is_none() && app.studios[rel].zoomansicht.pan == (0.0, 0.0));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Beispiel für den Studio-Rahmen (Muster): Platzhalterinhalt in allen sechs Zonen
    fn muster_demo(pal: Pal, k: vc_gui_ui::muster::Klasse) -> Element<'static, Msg> {
        use vc_gui_ui::muster as m;
        let knopf_l = |t: &str| -> Element<'static, Msg> { button(text(t.to_string()).size(12.5).color(pal.text)).padding([4, 10]).height(m::KNOPF_HOEHE).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: m::RADIUS_KNOPF.into() }, text_color: pal.text, ..Default::default() }).into() };
        let kopf = m::kopfzeile(pal, k, "SPR", "hero.vcsprite", true, "Frame 0 von 2 · 12×21 · Multicolor", vec![knopf_l("Exportieren …"), knopf_l("Speichern"), knopf_l("⋯")]);
        let leiste = m::leiste(pal, row![knopf_l("Stift"), knopf_l("Linie"), knopf_l("Füllen"), m::trenner(pal), knopf_l("↶"), knopf_l("↷"), m::trenner(pal), knopf_l("Transformieren ▾"), space::horizontal().width(Length::Fill), knopf_l("−"), knopf_l("+"), knopf_l("Einpassen")].spacing(m::abstand::ENG).align_y(Alignment::Center).into());
        let liste: Element<Msg> = iced::widget::column![text("Frames · 2").size(m::schrift::KARTE).color(pal.mute), knopf_l("Frame 0"), knopf_l("Frame 1")].spacing(m::abstand::ENG).into();
        let flaeche: Element<Msg> = container(text("Zeichenfläche (eingepasst, kein Balken)").size(12.5).color(pal.mute)).center(Length::Fill).into();
        let seite = m::seitenspalte(k, vec![
            m::karte(pal, "Werkzeug", "✎", "fürs Zeichnen", Some((true, Msg::StudioTick)), m::zeile(pal, "Zeichenfarbe", m::segment(pal, &[("0", "0"), ("1", "1"), ("2", "2"), ("3", "3")], "1", |_| Msg::StudioTick))),
            m::karte(pal, "Dokument", "▤", "fürs ganze Frame", Some((true, Msg::StudioTick)), m::zeile(pal, "Modus", m::segment(pal, &[("hires", "Hires"), ("multi", "Multicolor")], "multi", |_| Msg::StudioTick))),
            m::karte(pal, "Info", "ⓘ", "", Some((false, Msg::StudioTick)), text("zugeklappt").size(12).into()),
        ]);
        let fuss = m::fusszeile(pal, "Zelle 3,4 · Zoom Einpassen · 16×", text("? Tasten").size(11).color(pal.mute).into());
        m::studio_rahmen(pal, k, m::Studio { liste_breite: m::LISTE_BREITE, kopf, leiste, liste: Some(liste), flaeche, seite, fuss })
    }
    #[test] #[ignore] fn bild_muster_rahmen() {
        use vc_gui_ui::muster::Klasse;
        for (name, k, b) in [("breit", Klasse::Breit, 1000u32), ("mittel", Klasse::Mittel, 760), ("schmal", Klasse::Schmal, 600)] { foto_element(muster_demo(Pal::dark(), k), b, 560, Pal::dark().bg, &format!("/tmp/vc-muster-{name}.png")) }
    }
    #[test] fn sprite_studio_zeichnet_mit_der_maus_macht_rueckgaengig_und_speichert() {
        use iced::{mouse, Event, Point};
        let (mut app, tmp) = studio_app();
        assert!(app.studios.contains_key("hero.vcsprite") && app.editoren.is_empty(), "Sprite öffnet im Studio, nicht im Texteditor"); assert_eq!(app.name().as_deref(), Some("hero.vcsprite"));
        let _ = simuliere(&mut app, &[]); assert!(!app.studios["hero.vcsprite"].doc.dirty());
        // Klick und Ziehen mitten auf der Zeichenfläche: Pixel werden gesetzt, Änderung ist ungespeichert
        let p0 = Point::new(730.0, 380.0); let p1 = Point::new(770.0, 380.0);
        let msgs = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p0 }), p0), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p0), (Event::Mouse(mouse::Event::CursorMoved { position: p1 }), p1), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p1)]);
        assert!(msgs.iter().any(|m| matches!(m, Msg::StudioMaus(_, studio::Maus::Unten { .. }))), "die Zeichenfläche meldet den Klick: {msgs:?}");
        if !app.studios["hero.vcsprite"].doc.dirty() { let _ = app.update(Msg::StudioAkt("hero.vcsprite".into(), studio::Aktion::Wert(2))); let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p0), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p0)]); }
        assert!(app.studios["hero.vcsprite"].doc.dirty(), "gezeichnet");
        // Rückgängig (Befehlstabelle) stellt den Stand wieder her, Wiederholen setzt ihn zurück
        let _ = app.befehl(Cmd::Rueckgaengig); assert!(!app.studios["hero.vcsprite"].doc.dirty(), "Cmd+Z macht rückgängig"); let _ = app.befehl(Cmd::Wiederholen); assert!(app.studios["hero.vcsprite"].doc.dirty());
        // Werkzeugwahl per Einzeltaste (nur ohne Fokus in einem Textfeld), Speichern per Befehl
        let _ = app.update(Msg::Taste(iced::keyboard::Key::Character("f".into()), iced::keyboard::Modifiers::empty())); assert_eq!(app.studios["hero.vcsprite"].doc.tool, vc_ide_model::pixel::Tool::Fill);
        let _ = app.befehl(Cmd::Speichern); assert!(!app.studios["hero.vcsprite"].doc.dirty(), "gespeichert"); assert!(std::fs::read_to_string(tmp.join("hero.vcsprite")).unwrap().contains("frames"));
        // Frame-Verwaltung und Modus
        let n0 = app.studios["hero.vcsprite"].doc.frames.len(); let _ = app.update(Msg::StudioAkt("hero.vcsprite".into(), studio::Aktion::FrameDup)); assert_eq!(app.studios["hero.vcsprite"].doc.frames.len(), n0 + 1);
        let _ = app.update(Msg::StudioAkt("hero.vcsprite".into(), studio::Aktion::FrameLoeschen)); assert_eq!(app.studios["hero.vcsprite"].doc.frames.len(), n0);
        let _ = simuliere(&mut app, &[]); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn sprite_studio_rechte_taste_loescht_mit_allen_werkzeugen_ausser_auswahl() {
        use iced::{mouse, Event, Point}; use vc_ide_model::pixel::Tool;
        let rel = "hero.vcsprite"; let p0 = Point::new(730.0, 380.0); let p1 = Point::new(748.0, 380.0);
        let ereignis = |ev: mouse::Event, p: Point| (Event::Mouse(ev), p);
        for tool in [Tool::Pen, Tool::Erase, Tool::Fill, Tool::Line, Tool::Rect, Tool::Pick] {
            let (mut app, tmp) = studio_app(); let _ = simuliere(&mut app, &[]);
            let _ = app.update(Msg::StudioAkt(rel.into(), studio::Aktion::Wert(2))); app.studios.get_mut(rel).unwrap().doc.tool = Tool::Pen;
            let vor = app.studios[rel].doc.frames[0].clone();
            // links zeichnen (zwei Zellen ziehen) → es gibt Pixel mit Wert 1 unter dem Zeiger
            let _ = simuliere(&mut app, &[ereignis(mouse::Event::CursorMoved { position: p0 }, p0), ereignis(mouse::Event::ButtonPressed(mouse::Button::Left), p0), ereignis(mouse::Event::CursorMoved { position: p1 }, p1), ereignis(mouse::Event::ButtonReleased(mouse::Button::Left), p1)]);
            let gemalt = app.studios[rel].doc.frames[0].clone(); assert_ne!(gemalt, vor, "{tool:?}: links malt");
            let (undo0, tool_kopie) = (app.studios[rel].doc.dirty(), tool); assert!(undo0);
            app.studios.get_mut(rel).unwrap().doc.tool = tool;
            let msgs = simuliere(&mut app, &[ereignis(mouse::Event::CursorMoved { position: p0 }, p0), ereignis(mouse::Event::ButtonPressed(mouse::Button::Right), p0), ereignis(mouse::Event::CursorMoved { position: p1 }, p1), ereignis(mouse::Event::ButtonReleased(mouse::Button::Right), p1)]);
            assert!(msgs.iter().any(|m| matches!(m, Msg::StudioMaus(_, studio::Maus::Unten { rechts: true, .. }))), "{tool_kopie:?}: rechte Taste kommt an: {msgs:?}");
            assert!(app.kontext.is_none(), "{tool_kopie:?}: kein Menü");
            let danach = app.studios[rel].doc.frames[0].clone();
            let n = |f: &Vec<Vec<u8>>| f.iter().flatten().filter(|&&v| v != 0).count();
            match tool { Tool::Line | Tool::Rect | Tool::Fill => { assert_ne!(danach, gemalt, "{tool_kopie:?}: rechts ändert") } _ => { assert!(n(&danach) < n(&gemalt), "{tool_kopie:?}: rechts löscht ({} → {})", n(&gemalt), n(&danach)) } }
            assert!(!danach.iter().flatten().zip(gemalt.iter().flatten()).any(|(a, g)| *a != 0 && *a != *g), "{tool_kopie:?}: rechts malt nie neue Pixel");
            app.studios.get_mut(rel).unwrap().doc.undo(); assert_eq!(app.studios[rel].doc.frames[0], gemalt, "{tool_kopie:?}: ein Rückgängig-Schritt");
            let _ = std::fs::remove_dir_all(&tmp);
        }
        // Auswahlwerkzeug: Rechtsklick öffnet das Menü „Zeichenfläche“, löscht nichts
        let (mut app, tmp) = studio_app(); let _ = simuliere(&mut app, &[]); app.studios.get_mut(rel).unwrap().doc.tool = Tool::Select; let vor = app.studios[rel].doc.frames[0].clone();
        let _ = simuliere(&mut app, &[ereignis(mouse::Event::CursorMoved { position: p0 }, p0), ereignis(mouse::Event::ButtonPressed(mouse::Button::Right), p0)]);
        assert_eq!(app.kontext.as_ref().map(|k| k.art), Some(Art::Flaeche)); assert_eq!(app.studios[rel].doc.frames[0], vor); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn sprite_studio_animationen_und_zeichensatz_im_selben_studio() {
        let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite"; let akt = |app: &mut App, a: studio::Aktion| { let _ = app.update(Msg::StudioAkt(rel.into(), a)); };
        use vc_ide_model::pixel::AnimOp;
        akt(&mut app, studio::Aktion::Anim(AnimOp::New)); akt(&mut app, studio::Aktion::Anim(AnimOp::AddStep(1))); akt(&mut app, studio::Aktion::Anim(AnimOp::Delay(0, 9)));
        let l = app.studios[rel].doc.animationen().unwrap(); assert_eq!((l.len(), l[0].steps.len(), l[0].steps[0].1), (1, 2, 9));
        akt(&mut app, studio::Aktion::Anim(AnimOp::Move(0, 1))); assert_eq!(app.studios[rel].doc.animationen().unwrap()[0].steps[1].1, 9);
        akt(&mut app, studio::Aktion::Anim(AnimOp::Rename("lauf".into()))); assert_eq!(app.studios[rel].doc.animationen().unwrap()[0].name, "lauf");
        akt(&mut app, studio::Aktion::Anim(AnimOp::Rename("1 falsch".into()))); assert!(app.log.last().unwrap().contains("kein gültiger"), "ungültiger Name wird abgewiesen");
        let _ = simuliere(&mut app, &[]); let _ = app.befehl(Cmd::Speichern);
        assert!(std::fs::read_to_string(tmp.join(rel)).unwrap().contains("lauf"), "Animation gespeichert"); akt(&mut app, studio::Aktion::Anim(AnimOp::Delete)); assert!(app.studios[rel].doc.animationen().unwrap().is_empty());
        // Zeichensatz: dasselbe Studio, Zeichenraster statt Frame-Liste
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."); std::fs::copy(repo.join("examples/atari/gfx_demo/graphics/afont.vcchar"), tmp.join("afont.vcchar")).unwrap();
        app.oeffne("afont.vcchar"); assert_eq!(app.name().as_deref(), Some("afont.vcchar")); assert_eq!(app.studios["afont.vcchar"].doc.kind, "charset"); let _ = simuliere(&mut app, &[]);
        akt_c(&mut app, "afont.vcchar", studio::Aktion::Frame(5)); assert_eq!(app.studios["afont.vcchar"].doc.fi, 5);
        let _ = std::fs::remove_dir_all(&tmp);
        fn akt_c(app: &mut App, rel: &str, a: studio::Aktion) { let _ = app.update(Msg::StudioAkt(rel.into(), a)); }
    }
    #[test] fn export_dialog_vorschau_formate_speichern_und_include_einfuegen() {
        let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite";
        std::fs::write(tmp.join("main.asm"), ".target c64\n.org $0801\nstart: rts\n").unwrap(); app.projekt_neu_laden();
        app.oeffne("main.asm"); app.oeffne(rel); assert_eq!(app.name().as_deref(), Some(rel));
        assert!(app.nutzbar(Cmd::Exportieren)); let _ = app.befehl(Cmd::Exportieren);
        let d = app.export.as_ref().expect("Dialog offen"); assert_eq!((d.format.as_str(), d.cache.error.is_none(), d.cache.files.is_empty()), ("asm", true, false), "Vorschau des Quelltexts: {:?}", d.cache.error); assert!(d.cache.text.contains(".byte") || d.cache.text.contains("hero"), "{}", d.cache.text);
        let _ = simuliere(&mut app, &[]);
        // Formate: Konstanten und PRG brauchen eine Adresse; PRG ohne gültige Adresse ist ein Fehler, Hex und BASIC-DATA-Reiter rechnen
        let _ = app.update(Msg::ExportFormat("prg".into())); assert!(app.export.as_ref().unwrap().braucht_adresse() && app.export.as_ref().unwrap().cache.error.is_none());
        let _ = app.update(Msg::ExportAdresse("kaputt".into())); assert!(app.export.as_ref().unwrap().cache.error.is_some(), "ungültige Ladeadresse"); let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Speichern)); assert!(!tmp.join("build/export").exists(), "bei Fehler wird nichts geschrieben");
        let _ = app.update(Msg::ExportAdresse("$2000".into())); let _ = app.update(Msg::ExportTab("hex".into())); assert!(!app.export.as_ref().unwrap().cache.hex.is_empty(), "Hex-Vorschau");
        let _ = app.update(Msg::ExportFormat("data".into())); let _ = app.update(Msg::ExportTab("data".into())); assert!(matches!(app.export.as_ref().unwrap().data_tab, Some(Ok(_))), "{:?}", app.export.as_ref().unwrap().data_tab);
        let _ = simuliere(&mut app, &[]);
        // In Projekt speichern: Datei in build/export, Dialog bleibt offen
        let _ = app.update(Msg::ExportFormat("asm".into())); let _ = app.update(Msg::ExportName("held".into())); let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Speichern));
        let ausgabe: Vec<_> = std::fs::read_dir(tmp.join("build/export")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect(); assert!(ausgabe.iter().any(|n| n.ends_with(".asm")), "{ausgabe:?}"); assert!(app.export.is_some());
        // der Baum zeigt die Exportdateien in der Gruppe „Exportiert“ (der Bauordner selbst bleibt ausgeblendet), die Meldung nennt Ort und Gruppe
        assert!(app.exportiert.iter().any(|f| f.starts_with("build/export/") && f.ends_with(".asm")), "{:?}", app.exportiert); assert!(!app.namen.iter().any(|n| n.starts_with("build/")));
        assert!(app.export.as_ref().unwrap().meldung.contains("build/export/") && app.export.as_ref().unwrap().meldung.contains(t::GRUPPE_EXPORT), "{}", app.export.as_ref().unwrap().meldung);
        // In aktiven Code einfügen: `.include` des Exports im zuletzt benutzten Quelltext (Datei ist im Studio, also zählt der letzte Quelltext)
        let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Einfuegen)); assert!(app.export.is_none());
        assert!(app.editoren["main.asm"].text().contains(".include") && app.editoren["main.asm"].text().contains("build/export/"), "{}", app.editoren["main.asm"].text());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Punkt auf einer Zelle der Zeichenfläche des Studios `rel` (Zeiger über den Bereich schieben, bis das Studio eine Zelle meldet); unabhängig vom Layout
    fn zelle_punkt(app: &mut App, rel: &str, x0: f32, x1: f32, y0: f32, y1: f32) -> iced::Point {
        use iced::{mouse, Event, Point};
        let mut y = y0; while y <= y1 { let mut x = x0; while x <= x1 { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]); if app.studios.get(rel).is_some_and(|s| s.ueber.is_some()) { return p } x += 6.0 } y += 18.0 }
        panic!("keine Zelle der Zeichenfläche von {rel} im Bereich gefunden")
    }
    /// Raster eines Bildschirm-Studios: (x0, y0, Kantenlänge) der Zelle 0,0; gefunden durch Bewegen des Zeigers (unabhängig vom Layout): erst ein Treffer nahe der Mitte, dann die Zellgrenzen
    fn screen_gitter(app: &mut App, rel: &str) -> (f32, f32, f32) {
        use iced::{mouse, Event, Point};
        let mut hover = |app: &mut App, x: f32, y: f32| -> Option<(usize, usize)> { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]); app.screens.get(rel).and_then(|s| s.ueber) };
        let _ = simuliere(app, &[]); let _ = simuliere(app, &[]);
        let mut treffer = None; for (dx, dy) in [(0.0, 0.0), (60.0, 0.0), (-60.0, 0.0), (0.0, 60.0), (0.0, -60.0), (120.0, 60.0), (-120.0, -60.0), (120.0, -60.0), (-120.0, 60.0)] { if let Some(c) = hover(app, 600.0 + dx, 380.0 + dy) { treffer = Some((600.0 + dx, 380.0 + dy, c)); break } }
        let (px, py, (cx, cy)) = treffer.expect("Zelle der Zeichenfläche gefunden");
        let (mut x, mut y) = (px, py); while hover(app, x - 2.0, py) == Some((cx, cy)) { x -= 2.0 } while hover(app, px, y - 2.0) == Some((cx, cy)) { y -= 2.0 }
        let kante = app.screens[rel].skala.get() as f32 * 8.0; (x - cx as f32 * kante, y - cy as f32 * kante, kante)
    }
    /// Raster eines Kachelsatz-Studios: (x0, y0, Kantenlänge) der Zelle 0; gefunden durch Bewegen des Zeigers (unabhängig vom Layout)
    fn tile_gitter(app: &mut App, rel: &str) -> (f32, f32, f32) {
        use iced::{mouse, Event, Point};
        let mut hover = |app: &mut App, x: f32, y: f32| -> Option<usize> { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]); app.tilesets.get(rel).and_then(|s| s.ueber) };
        let _ = simuliere(app, &[]); let _ = simuliere(app, &[]);
        let mut treffer = None; for (dx, dy) in [(0.0, 0.0), (40.0, 0.0), (-40.0, 0.0), (0.0, 40.0), (0.0, -40.0), (80.0, 40.0), (-80.0, -40.0), (80.0, -40.0), (-80.0, 40.0)] { if let Some(c) = hover(app, 600.0 + dx, 380.0 + dy) { treffer = Some((600.0 + dx, 380.0 + dy, c)); break } }
        let (px, py, k) = treffer.expect("Zelle der Kachel gefunden"); let tw = app.tilesets[rel].doc.ts.tw;
        let (mut x, mut y) = (px, py); while hover(app, x - 2.0, py) == Some(k) { x -= 2.0 } while hover(app, px, y - 2.0) == Some(k) { y -= 2.0 }
        let kante = app.tilesets[rel].skala.get() as f32 * 8.0; (x - (k % tw) as f32 * kante, y - (k / tw) as f32 * kante, kante)
    }
    /// Raster eines Bank-Studios: (x0, y0, Zellkante) der Zelle 0,0 (hinter dem Rand); gefunden durch Bewegen des Zeigers (unabhängig vom Layout)
    fn bank_gitter(app: &mut App, rel: &str) -> (f32, f32, f32) {
        use iced::{mouse, Event, Point};
        let mut hover = |app: &mut App, x: f32, y: f32| -> Option<(i32, i32)> { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]); app.banken.get(rel).and_then(|s| s.ueber) };
        let _ = simuliere(app, &[]); let _ = simuliere(app, &[]);
        let mut treffer = None; for (dx, dy) in [(0.0, 0.0), (40.0, 0.0), (-40.0, 0.0), (0.0, 40.0), (0.0, -40.0), (80.0, 40.0), (-80.0, -40.0), (80.0, -40.0), (-80.0, 40.0)] { if let Some(c) = hover(app, 600.0 + dx, 380.0 + dy) { treffer = Some((600.0 + dx, 380.0 + dy, c)); break } }
        let (px, py, (cx, cy)) = treffer.expect("Zelle des Screens gefunden");
        let (mut x, mut y) = (px, py); while hover(app, x - 2.0, py) == Some((cx, cy)) { x -= 2.0 } while hover(app, px, y - 2.0) == Some((cx, cy)) { y -= 2.0 }
        let kante = app.banken[rel].skala.get() as f32 * 8.0; (x - cx as f32 * kante, y - cy as f32 * kante, kante)
    }
    fn kopie_von_beispiel(rel: &str) -> std::path::PathBuf {
        static Z: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let quelle = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel).canonicalize().unwrap();
        let ziel = std::env::temp_dir().join(format!("vcgui-bsp-{}-{}", std::process::id(), Z.fetch_add(1, std::sync::atomic::Ordering::SeqCst))); let _ = std::fs::remove_dir_all(&ziel);
        fn kopiere(a: &std::path::Path, b: &std::path::Path) { std::fs::create_dir_all(b).unwrap(); for e in std::fs::read_dir(a).unwrap().flatten() { let (p, q) = (e.path(), b.join(e.file_name())); if p.is_dir() { kopiere(&p, &q) } else { std::fs::copy(&p, &q).unwrap(); } } }
        kopiere(&quelle, &ziel); ziel
    }
    #[test] fn export_in_aktiven_code_einfuegen_im_beispielprojekt_sprite_demo_mit_allen_ablehnungen() {
        let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe();
        // Bedienung wie in der Handprüfung: Projekt öffnen, sprite_demo.asm ist offen, Studio öffnen, Export, Knopf
        assert!(app.projekt_oeffnen(tmp.clone())); assert_eq!(app.name().as_deref(), Some("sprite_demo.asm"));
        app.oeffne("graphics/hero.vcsprite"); assert!(app.studios.contains_key("graphics/hero.vcsprite"));
        let _ = app.befehl(Cmd::Exportieren); assert!(app.export.is_some());
        let vorher = app.editoren["sprite_demo.asm"].text(); assert!(vorher.contains(".include \"build/export/hero.asm\""), "das Beispiel bindet den Export schon ein");
        let _ = simuliere(&mut app, &[]);
        // schon eingebunden: sichtbare Ablehnung (Dialog bleibt offen und nennt den Grund, Protokoll auch), nichts doppelt eingefügt
        let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Einfuegen));
        assert!(app.export.as_ref().is_some_and(|d| d.meldung.contains("schon eingebunden")), "Meldung im Dialog: {:?}", app.export.as_ref().map(|d| d.meldung.clone()));
        assert!(app.log.last().is_some_and(|l| l.contains("schon eingebunden")), "{:?}", app.log.last()); assert_eq!(app.editoren["sprite_demo.asm"].text(), vorher); assert_eq!(app.schublade, 0);
        // ohne die vorhandene Zeile: eingefügt, Dialog schließt, Protokoll meldet; Pfad relativ zur .asm
        { let e = app.editoren.get_mut("sprite_demo.asm").unwrap(); let t = e.text().replace(".include \"build/export/hero.asm\"\n", ""); *e = Editor::new(&t); }
        app.oeffne("graphics/hero.vcsprite"); let _ = app.befehl(Cmd::Exportieren); let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Einfuegen));
        let nachher = app.editoren["sprite_demo.asm"].text();
        assert!(app.export.is_none(), "Dialog schließt: {:?}", app.log.last()); assert!(nachher.contains(".include \"build/export/hero.asm\""), "Include steht in sprite_demo.asm; Protokoll: {:?}", app.log);
        assert!(app.log.last().is_some_and(|l| l.starts_with("In sprite_demo.asm eingefügt: .include")), "{:?}", app.log.last());
        // Wiederholung: jetzt wieder schon eingebunden
        app.oeffne("graphics/hero.vcsprite"); let _ = app.befehl(Cmd::Exportieren); let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Einfuegen));
        assert_eq!(app.editoren["sprite_demo.asm"].text(), nachher, "nicht doppelt eingefügt"); assert!(app.export.is_some() && app.log.last().is_some_and(|l| l.contains("schon eingebunden")), "{:?}", app.log.last());
        // Kein Quelltext offen: verständliche Meldung statt Stille
        let tmp2 = kopie_von_beispiel("examples/c64/sprite_demo"); let (mut app2, _) = App::neu(); app2.einst = Einstellungen::default();
        assert!(app2.projekt_oeffnen(tmp2.join("graphics/hero.vcsprite"))); app2.editoren.clear(); app2.tabs.retain(|t| matches!(t, Tab::Datei(n) if n.ends_with(".vcsprite"))); app2.letzte_asm = None; app2.oeffne("graphics/hero.vcsprite");
        let _ = app2.befehl(Cmd::Exportieren); let _ = app2.update(Msg::ExportAus(exportdlg::ExportAus::Einfuegen));
        assert!(app2.log.last().is_some_and(|l| l.contains("Kein Quelltext")), "{:?}", app2.log.last()); assert!(app2.export.as_ref().is_some_and(|d| d.meldung.contains("Kein Quelltext")), "auch im Dialog");
        let _ = (std::fs::remove_dir_all(&tmp), std::fs::remove_dir_all(&tmp2));
    }
    #[test] fn abgelegte_dateien_fragen_nach_und_werden_uebernommen() {
        let (mut app, tmp) = studio_app(); let ext = std::env::temp_dir().join(format!("vcgui-ablage-{}", std::process::id())); let _ = std::fs::remove_dir_all(&ext); std::fs::create_dir_all(&ext).unwrap();
        std::fs::write(ext.join("neu.asm"), "nop\n").unwrap(); std::fs::write(ext.join("hero.vcsprite"), "{}").unwrap(); // gleicher Name wie im Projekt, anderer Inhalt: Konflikt
        // Datei aus dem Projekt: wird wie bisher geöffnet; von außen: Rückfrage
        let _ = app.update(Msg::Abgelegt(tmp.join("hero.vcsprite"))); assert!(app.import.is_none());
        let _ = app.update(Msg::Abgelegt(ext.join("neu.asm"))); let _ = app.update(Msg::Abgelegt(ext.join("hero.vcsprite")));
        let d = app.import.as_ref().expect("Rückfrage"); assert_eq!(d.zeilen.len(), 2); let _ = simuliere(&mut app, &[]);
        assert_eq!(vc_ide::importfiles::target_state(&tmp, &app.import.as_ref().unwrap().zeilen[1].plan), vc_ide::importfiles::Target::Clash, "gleicher Name, anderer Inhalt");
        let _ = app.update(Msg::ImportKonflikt(1, vc_ide::importfiles::Conflict::Skip)); let _ = app.update(Msg::ImportName(0, "mein.asm".into()));
        let _ = app.update(Msg::ImportAbbruch); assert!(app.import.is_none() && !app.namen.iter().any(|n| n.ends_with("mein.asm")), "Abbrechen übernimmt nichts");
        let _ = app.update(Msg::Abgelegt(ext.join("neu.asm"))); let _ = app.update(Msg::Abgelegt(ext.join("hero.vcsprite")));
        let _ = app.update(Msg::ImportName(0, "mein.asm".into())); let _ = app.update(Msg::ImportKonflikt(1, vc_ide::importfiles::Conflict::Skip)); let _ = app.update(Msg::ImportOk);
        let mein = app.namen.iter().find(|n| n.ends_with("mein.asm")).cloned().expect("mein.asm im Projekt"); assert!(app.import.is_none() && tmp.join(&mein).is_file(), "übernommen nach {mein}"); assert_ne!(std::fs::read_to_string(tmp.join("hero.vcsprite")).unwrap(), "{}", "Überspringen lässt die vorhandene Datei in Ruhe");
                let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&ext);
    }
    fn dateibaum(root: &std::path::Path) -> Vec<(String, Vec<u8>)> {
        fn lauf(root: &std::path::Path, d: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) { for e in std::fs::read_dir(d).unwrap().flatten() { let p = e.path(); if p.file_name().is_some_and(|n| n == "konf") { continue } if p.is_dir() { lauf(root, &p, out) } else { out.push((p.strip_prefix(root).unwrap().to_string_lossy().into_owned(), std::fs::read(&p).unwrap())) } } }
        let mut v = vec![]; lauf(root, root, &mut v); v.sort(); v
    }
    #[test] fn import_zustandsmatrix_vorgaben_zaehler_und_nichts_tun_bei_gleichem_inhalt() {
        use vc_ide::importfiles::{Conflict, Target};
        let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone()));
        let ext = std::env::temp_dir().join(format!("vcgui-matrix-{}", std::process::id())); let _ = std::fs::remove_dir_all(&ext); std::fs::create_dir_all(ext.join("mix")).unwrap();
        std::fs::create_dir_all(tmp.join("doc")).unwrap(); std::fs::write(tmp.join("doc/cli.md"), "# Kommandozeile\n\nBeispieltext für den Import-Test (liegt nur in der Kopie, nicht im Beispielprojekt).\n").unwrap(); let cli = std::fs::read(tmp.join("doc/cli.md")).unwrap(); assert!(!cli.is_empty());
        // 1) nur „gleicher Inhalt“: Haken aus, Zähler 0, Knopf ohne Wirkung, Dateisystem/Projektdatei/Baum unverändert
        std::fs::write(ext.join("cli.md"), &cli).unwrap();
        let (fs0, namen0, vcprj0) = (dateibaum(&tmp), app.namen.clone(), std::fs::read_to_string(tmp.join("sprite_demo.vcprj")).unwrap());
        app.baum_ziel = "doc".into(); // Ablage in den im Baum gewählten Ordner
        let _ = app.update(Msg::Abgelegt(ext.join("cli.md"))); let d = app.import.as_ref().expect("Rückfrage");
        assert_eq!(importfiles_zustand(&app, 0), Target::Same); assert!(!d.zeilen[0].an, "Vorgabe: Haken aus"); assert_eq!(importdlg::wirksam(&app.root, d), 0);
        let _ = app.update(Msg::ImportAn(0, true)); assert!(!app.import.as_ref().unwrap().zeilen[0].an, "lässt sich nicht anhaken");
        let _ = app.update(Msg::ImportOk);
        { let jetzt = dateibaum(&tmp); let (a, b): (Vec<_>, Vec<_>) = (fs0.iter().map(|f| f.0.clone()).collect(), jetzt.iter().map(|f| f.0.clone()).collect()); assert_eq!(a, b, "Dateiliste unverändert"); for (x, y) in fs0.iter().zip(&jetzt) { assert!(x.1 == y.1, "Inhalt von {} unverändert", x.0) } } assert_eq!(app.namen, namen0, "Baum unverändert"); assert_eq!(std::fs::read_to_string(tmp.join("sprite_demo.vcprj")).unwrap(), vcprj0, "Projektdatei unverändert");
        // ausdrücklich „als Kopie unter neuem Namen“: neuer Name, Haken an, wird angelegt; Original unberührt
        let _ = app.update(Msg::Abgelegt(ext.join("cli.md"))); if app.import.is_none() { panic!("Rückfrage") }
        let _ = app.update(Msg::ImportKopie(0)); { let d = app.import.as_ref().unwrap(); assert!(d.zeilen[0].an && d.zeilen[0].plan.name != "cli.md", "{}", d.zeilen[0].plan.name); assert_eq!(importfiles_zustand(&app, 0), Target::Free); }
        let _ = app.update(Msg::ImportOk); assert!(app.import.is_none()); assert!(tmp.join("doc/cli_2.md").is_file() && std::fs::read(tmp.join("doc/cli.md")).unwrap() == cli);
        // 2) Ordner mit gemischten Zuständen: neu (an), Name belegt mit anderem Inhalt (an, Umbenennen), gleicher Inhalt (aus)
        std::fs::write(ext.join("mix/neu.md"), "# neu\n").unwrap(); std::fs::write(ext.join("mix/cli.md"), &cli).unwrap();
        let belegt = tmp.join("doc/notiz.md"); std::fs::write(&belegt, "alt\n").unwrap(); std::fs::write(ext.join("mix/notiz.md"), "ANDERS\n").unwrap(); app.projekt_neu_laden();
        let _ = app.update(Msg::Abgelegt(ext.join("mix")));
        let zust: Vec<(String, bool, Target, Conflict)> = { let d = app.import.as_ref().expect("Rückfrage Ordner"); d.zeilen.iter().enumerate().map(|(i, z)| (z.plan.name.clone(), z.an, importfiles_zustand(&app, i), z.plan.conflict)).collect() };
        let von = |n: &str| zust.iter().find(|z| z.0 == n).cloned().unwrap_or_else(|| panic!("{n} fehlt: {zust:?}"));
        assert_eq!((von("neu.md").1, von("neu.md").2), (true, Target::Free)); assert_eq!((von("notiz.md").1, von("notiz.md").2, von("notiz.md").3), (true, Target::Clash, Conflict::Rename), "Konflikt: Vorgabe Umbenennen"); assert_eq!((von("cli.md").1, von("cli.md").2), (false, Target::Same));
        assert_eq!(importdlg::wirksam(&app.root, app.import.as_ref().unwrap()), 2, "Zähler zählt nur echte Übernahmen");
        let vcprj1 = std::fs::read_to_string(tmp.join("sprite_demo.vcprj")).unwrap();
        let _ = app.update(Msg::ImportOk);
        assert!(tmp.join("doc/neu.md").is_file() && tmp.join("doc/notiz_2.md").is_file() && std::fs::read_to_string(&belegt).unwrap() == "alt\n", "neu angelegt, belegter Name umbenannt, Original bleibt");
        assert_eq!(std::fs::read(tmp.join("doc/cli.md")).unwrap(), cli); let vcprj2 = std::fs::read_to_string(tmp.join("sprite_demo.vcprj")).unwrap();
        let pf = vc_ide::projfile::ProjectFile::load(&tmp.join("sprite_demo.vcprj")).unwrap(); let mut dateien: Vec<&str> = pf.assets.iter().map(|a| a.file.as_str()).collect(); let n = dateien.len(); dateien.sort(); dateien.dedup(); assert_eq!(dateien.len(), n, "keine doppelten Einträge in assets"); let _ = (vcprj1, vcprj2);
        let _ = (std::fs::remove_dir_all(&tmp), std::fs::remove_dir_all(&ext));
    }
    fn importfiles_zustand(app: &App, i: usize) -> vc_ide::importfiles::Target { vc_ide::importfiles::target_state(&app.root, &app.import.as_ref().unwrap().zeilen[i].plan) }
    fn geteilt_app() -> (App, std::path::PathBuf) {
        let (mut app, tmp) = studio_app(); std::fs::copy(tmp.join("hero.vcsprite"), tmp.join("zwei.vcsprite")).unwrap();
        std::fs::write(tmp.join("main.asm"), ".target c64\n.org $0801\nstart: rts\n").unwrap(); app.projekt_neu_laden();
        app.oeffne("main.asm"); app.oeffne("hero.vcsprite"); app.oeffne("zwei.vcsprite"); app.layout = Layout::vorgabe(); (app, tmp)
    }
    #[test] #[ignore] fn bild_geteilt() { let (mut app, tmp) = geteilt_app(); let _ = app.befehl(Cmd::Teilen); eprintln!("rechts={} links={:?} aktiv={} andere={:?}", app.fokus_rechts, app.tabs, app.aktiv, app.andere); foto(&mut app, &[], "/tmp/vc-geteilt.png"); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] fn fokus_folgt_dem_klick_im_geteilten_dokumentenfeld() {
        use iced::{mouse, Event, Point};
        let (mut app, tmp) = geteilt_app(); let _ = app.befehl(Cmd::Teilen); let _ = simuliere(&mut app, &[]);
        let klick = |app: &mut App, x: f32, y: f32| { let p = Point::new(x, y); simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]) };
        // Fokus im Quelltext (links), dann „Exportieren …“ im Studio rechts: der Dialog gehört zu DIESEM Studio
        let _ = klick(&mut app, 500.0, 300.0); assert_eq!(app.name().as_deref(), Some("main.asm"), "Fokus im Quelltext");
        let (x, y) = ort(&mut app, "Speichern"); let _ = klick(&mut app, x, y); let _ = app.befehl(Cmd::Exportieren); // schmal: die Kopfzeile zeigt nur Speichern, Export über die Befehlspalette
        assert_eq!(app.export.as_ref().map(|d| d.rel.clone()).as_deref(), Some("zwei.vcsprite"), "Export für das angeklickte Studio; Protokoll: {:?}", app.log.last()); assert_eq!(app.name().as_deref(), Some("zwei.vcsprite"), "Fokus folgt dem Klick");
        let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Schliessen));
        // zwei Studios nebeneinander: Klick auf die Fläche des rechten wirkt dort, Cmd+Z/Cmd+S auf die aktive Gruppe
        let _ = klick(&mut app, 316.0, 68.0); assert_eq!(app.name().as_deref(), Some("hero.vcsprite"), "Reiter hero links aktiv");
        let _ = simuliere(&mut app, &[]);
        let zp = zelle_punkt(&mut app, "zwei.vcsprite", 700.0, 1240.0, 200.0, 600.0); let _ = klick(&mut app, zp.x, zp.y); assert!(app.studios["zwei.vcsprite"].doc.dirty() && !app.studios["hero.vcsprite"].doc.dirty(), "Zeichnen wirkt auf das rechte Studio"); assert_eq!(app.name().as_deref(), Some("zwei.vcsprite"));
        let _ = app.befehl(Cmd::Rueckgaengig); assert!(!app.studios["zwei.vcsprite"].doc.dirty(), "Cmd+Z wirkt auf die aktive Gruppe");
        let _ = simuliere(&mut app, &[]); let _ = klick(&mut app, 540.0, 340.0);
        assert_eq!(app.name().as_deref(), Some("hero.vcsprite"), "Klick in das linke Studio macht es aktiv"); assert!(app.studios["hero.vcsprite"].doc.dirty() || app.studios["hero.vcsprite"].doc.kann_undo(), "Zeichnen wirkt auf das linke Studio");
        assert!(!app.studios["zwei.vcsprite"].doc.dirty(), "das rechte bleibt unberührt");
        // Studio-Knöpfe tragen ihr Ziel: Aktion für ein anderes Studio als das fokussierte wirkt dort und holt den Fokus
        let _ = app.update(Msg::StudioAkt("zwei.vcsprite".into(), studio::Aktion::Leeren)); assert_eq!(app.name().as_deref(), Some("zwei.vcsprite")); assert!(app.studios["zwei.vcsprite"].doc.dirty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    fn disk_app() -> (App, std::path::PathBuf) {
        static Z: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let tmp = std::env::temp_dir().join(format!("vcgui-disk-{}-{}", std::process::id(), Z.fetch_add(1, std::sync::atomic::Ordering::SeqCst))); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        let stub = [0x01u8, 0x08, 0x0B, 0x08, 0x0A, 0x00, 0x9E, b'2', b'0', b'6', b'1', 0x00, 0x00, 0x00, 0xA9, 0x00, 0x60];
        let mut d = vc_disk::D64::blank("TEST", "01").unwrap(); d.add_prg("STUB", &stub).unwrap(); d.add_prg("MC", &[0x00, 0xC0, 0xEE, 0x20, 0xD0, 0x60]).unwrap(); d.add_prg("DATEN", &vec![5u8; 700]).unwrap();
        std::fs::write(tmp.join("spiel.d64"), d.to_bytes()).unwrap();
        let mut a = vc_disk::Atr::new_dos2(); a.dos2_add("AUTORUN.SYS", &[0xFF, 0xFF, 0, 0x20, 0x02, 0x20, 0xA9, 0x00, 0x60]).unwrap(); std::fs::write(tmp.join("a.atr"), a.to_bytes()).unwrap();
        std::fs::write(tmp.join("main.asm"), ".target c64\nstart: rts\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear();
        einstellungen::test_konfig(tmp.join("konf")); app.oeffne("main.asm"); app.oeffne("spiel.d64"); (app, tmp)
    }
    #[test] #[ignore] fn bild_disk_studio() { let (mut app, tmp) = disk_app(); let _ = app.update(Msg::DiskAkt("spiel.d64".into(), diskstudio::DAkt::Datei(0))); foto(&mut app, &[], "/tmp/vc-disk.png"); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] fn disk_studio_oeffnen_waehlen_pruefen_ablegen_speichern_uebernehmen_einlegen() {
        use iced::{mouse, Event, Point}; use diskstudio::DAkt;
        let (mut app, tmp) = disk_app(); let rel = "spiel.d64";
        assert!(app.disks.contains_key(rel) && app.name().as_deref() == Some(rel) && app.editoren.len() == 1, "Image öffnet im Disk-Studio, nicht im Texteditor");
        let _ = simuliere(&mut app, &[]);
        let klick = |app: &mut App, x: f32, y: f32| { let p = Point::new(x, y); simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]) };
        assert_eq!(app.disks[rel].files.len(), 3);
        // Verzeichnis: Klick auf den Namen wählt die Datei und springt im Inspektor an ihren Anfang
        let (x, y) = ort(&mut app, "\"MC\""); let _ = klick(&mut app, x, y); assert_eq!(app.disks[rel].sel_file, Some(1), "Klick auf MC wählt die Datei"); assert_eq!(app.disks[rel].cur, app.disks[rel].files[1].start);
        // Vormerken per Kästchen, dann Alle/Keine
        let (_, y) = ort(&mut app, "\"MC\""); let _ = klick(&mut app, 285.0, y); assert_eq!(app.disks[rel].anzahl_vorgemerkt(), 1, "Kästchen merkt vor");
        // Belegungskarte: Klick auf die Zelle der Datei DATEN wählt sie; Folgen/Zurück
        let ziel = app.disks[rel].files[2].start; let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Sektor(ziel))); assert_eq!(app.disks[rel].sel_file, Some(2));
        let n0 = app.disks[rel].history.len(); let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Folgen)); assert_eq!(app.disks[rel].history.len(), n0 + 1); let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Zurueck)); assert_eq!(app.disks[rel].cur, ziel);
        // Validieren: Bericht erscheint (keine Fehler), ✕ schließt ihn
        let (x, y) = ort(&mut app, "Validieren"); let _ = klick(&mut app, x, y); assert_eq!(app.disks[rel].report, Some(vec![]), "Validieren per Klick"); let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::ReportZu)); assert!(app.disks[rel].report.is_none());
        // Ablegen ins Fenster: Datei kommt auf die Diskette (kein Import-Dialog), Image ist ungespeichert, Reiter zeigt es
        std::fs::write(tmp.join("level.bin"), vec![9u8; 600]).unwrap(); let _ = app.update(Msg::Abgelegt(tmp.join("level.bin")));
        assert!(app.import.is_none() && app.disks[rel].dirty && app.disks[rel].files.len() == 4, "{:?}", app.disks[rel].note);
        // Einlegen erst nach dem Speichern; Speichern per Befehl schreibt das Image
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Einlegen)); assert!(aussen::aufgezeichnet().is_empty(), "ungespeichert: nichts eingelegt");
        let _ = app.befehl(Cmd::Speichern); assert!(!app.disks[rel].dirty); assert_eq!(vc_disk::AnyDisk::from_file_bytes("x.d64", &std::fs::read(tmp.join(rel)).unwrap()).unwrap().files().len(), 4, "Image auf der Platte enthält die Datei");
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Einlegen)); assert!(matches!(aussen::aufgezeichnet().as_slice(), [aussen::Aufruf::DiskEinlegen(p)] if p.ends_with(rel)), "Einlegen läuft über `aussen` und startet im Test nichts");
        // Übernahme ins Projekt: nur Vorgemerktes, BASIC als Quelltext
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Alle)); let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Uebernehmen));
        assert!(tmp.join("import/stub.bas").is_file() && tmp.join("import/mc.prg").is_file() && tmp.join("import/daten.prg").is_file(), "{:?}", app.log); assert!(app.namen.iter().any(|n| n == "import/stub.bas"), "Baum zeigt die Übernahme");
        // ungültiger Zielordner: nichts geschieht
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Zielordner("../draussen".into()))); let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Uebernehmen)); assert!(!tmp.parent().unwrap().join("draussen").exists());
        // Atari-Image und Projektwechsel
        app.oeffne("a.atr"); assert!(app.disks.contains_key("a.atr") && app.disks["a.atr"].atari() && app.disks["a.atr"].files.len() == 1); let _ = simuliere(&mut app, &[]);
        assert!(app.studio_a11y().is_some_and(|(n, w)| n.contains("a.atr") && w.contains("1 Dateien")), "{:?}", app.studio_a11y());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn neue_diskette_aus_dem_dialog_alle_formate_oeffnet_im_disk_studio() {
        let (mut app, tmp) = disk_app();
        for (k, ende) in [("d64", ".d64"), ("d71", ".d71"), ("d81", ".d81"), ("atr", ".atr")] {
            let _ = app.update(Msg::BaumNeu("disks".into(), NeuArt::Asm)); let _ = app.update(Msg::BaumArtWahl(NeuArt::Diskette(k))); let _ = app.update(Msg::BaumText("neu".into())); let _ = app.update(Msg::BaumOk);
            assert!(app.baum.is_none(), "{k}: {:?}", app.baum.as_ref().map(|b| b.fehler.clone())); let rel = format!("disks/neu{ende}");
            assert!(tmp.join(&rel).is_file() && app.disks.contains_key(&rel) && app.name().as_deref() == Some(rel.as_str()), "{rel} angelegt und im Studio offen");
            let d = &app.disks[&rel]; assert!(d.disk.is_ok() && !d.dirty && d.files.is_empty() && d.an.problems.is_empty(), "{rel}: leer, gültig, gespeichert");
            if k != "atr" { assert_eq!(vc_ide_model::diskdoc::petscii_text(&d.disk.as_ref().unwrap().title()), "NEU", "Disketten-Name aus dem Dateinamen"); }
            // gleicher Name nochmal: Fehler im Dialog, nichts überschrieben
            let _ = app.update(Msg::BaumNeu("disks".into(), NeuArt::Asm)); let _ = app.update(Msg::BaumArtWahl(NeuArt::Diskette(k))); let _ = app.update(Msg::BaumText("neu".into())); let _ = app.update(Msg::BaumOk);
            assert!(app.baum.as_ref().is_some_and(|b| b.fehler.contains("gibt es schon")), "{k}"); let _ = app.update(Msg::BaumAbbruch);
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn palette_folgt_der_videonorm_des_projekts_umschalter_nur_ansicht() {
        use studio::Ansicht;
        let tmp = kopie_von_beispiel("examples/atari/pmg_ship"); einstellungen::test_konfig(tmp.join("konf"));
        let vcprj = tmp.join("pmg_ship.vcprj"); let text = std::fs::read_to_string(&vcprj).unwrap(); assert!(text.contains("\"variant\": \"pal\"")); std::fs::write(&vcprj, text.replace("\"variant\": \"pal\"", "\"variant\": \"ntsc\"")).unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone())); let rel = "graphics/ship.vcpmg"; app.oeffne(rel);
        let ziel = app.studios[rel].doc.target.clone(); assert_eq!(ziel, "atari8");
        // Projekt mit variant ntsc → NTSC-Palette und Benennung „aus dem Projekt“
        assert_eq!(app.norm_fuer(&ziel, Ansicht::Projekt), "ntsc"); assert_eq!(app.palettenzeile(&ziel, Ansicht::Projekt), "Palette: Atari GTIA (NTSC, aus dem Projekt)");
        // PAL und NTSC unterscheiden sich für mehrere Farbwerte (Grau $00/$0F kann gleich sein, Farben nicht)
        let verschieden = [0x34usize, 0x96, 0xC8, 58, 134, 196].iter().filter(|v| app.farben.farbe(&ziel, "pal", **v) != app.farben.farbe(&ziel, "ntsc", **v)).count(); assert!(verschieden >= 4, "PAL/NTSC unterscheiden sich: {verschieden}");
        // Umschalter: nur Ansicht; Datei, Projektdatei und Farbwerte bleiben gleich, die Wahl gehört dem Reiter
        let (datei0, proj0, werte0) = (std::fs::read(tmp.join(rel)).unwrap(), std::fs::read(&vcprj).unwrap(), app.studios[rel].doc.farbe("P0", 14));
        let _ = app.update(Msg::StudioAkt(rel.into(), studio::Aktion::Ansicht(Ansicht::Pal)));
        assert_eq!(app.studios[rel].ansicht, Ansicht::Pal); assert_eq!(app.norm_fuer(&ziel, Ansicht::Pal), "pal");
        assert_eq!(app.palettenzeile(&ziel, Ansicht::Pal), "Palette: Atari GTIA (PAL) – Ansicht weicht vom Projekt ab (NTSC)");
        assert!(!app.studios[rel].doc.dirty(), "Umschalten ändert die Daten nicht"); assert_eq!(app.studios[rel].doc.farbe("P0", 14), werte0, "Farbwert der Datei gleich");
        let _ = app.befehl(Cmd::AlleSpeichern); assert_eq!(std::fs::read(tmp.join(rel)).unwrap(), datei0, "Datei unverändert"); assert_eq!(std::fs::read(&vcprj).unwrap(), proj0, "Projektdatei unverändert");
        let _ = app.update(Msg::StudioAkt(rel.into(), studio::Aktion::Ansicht(Ansicht::Projekt))); assert_eq!(app.palettenzeile(&ziel, app.studios[rel].ansicht), "Palette: Atari GTIA (NTSC, aus dem Projekt)");
        // Anzeige im Bild: Spielerfarbe (Frame-Vorschau, Fläche, Farbwähler benutzen dieselbe Tabelle): Farbe von P0 unterscheidet sich zwischen den Ansichten
        let rgb = |a: Ansicht| app.farben.farbe(&ziel, app.norm_fuer(&ziel, a), 58); assert_ne!(rgb(Ansicht::Pal), rgb(Ansicht::Ntsc)); assert_eq!(rgb(Ansicht::Projekt), rgb(Ansicht::Ntsc));
        // ohne Projektvariante (PAL) und Systeme ohne getrennte Tabellen: keine Anzeige der Videonorm, kein Umschalter
        assert!(app.farben.hat_varianten("atari8") && !app.farben.hat_varianten("c64") && !app.farben.hat_varianten("vic20"));
        assert_eq!(app.palettenzeile("c64", Ansicht::Ntsc), "Palette: C64"); assert_eq!(studio::projektnorm(None, "atari8"), "pal");
        // Zeichensatz-Studio folgt derselben Regel (Atari-Zeichensatz im ntsc-Projekt)
        let tmp2 = kopie_von_beispiel("examples/atari/gfx_demo"); let v2 = tmp2.join("gfx_demo.vcprj"); if let Ok(t) = std::fs::read_to_string(&v2) { std::fs::write(&v2, t.replace("\"variant\": \"pal\"", "\"variant\": \"ntsc\"")).unwrap() }
        let (mut app2, _) = App::neu(); app2.einst = Einstellungen::default(); assert!(app2.projekt_oeffnen(tmp2.clone())); app2.oeffne("graphics/afont.vcchar"); assert_eq!(app2.studios["graphics/afont.vcchar"].doc.target, "atari8"); assert_eq!(app2.norm_fuer("atari8", Ansicht::Projekt), "ntsc");
        let _ = (std::fs::remove_dir_all(&tmp), std::fs::remove_dir_all(&tmp2));
    }
    fn screen_app() -> (App, std::path::PathBuf) {
        let tmp = kopie_von_beispiel("examples/c64/gfx_demo"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone())); app.oeffne("graphics/title.vcscreen"); (app, tmp)
    }
    #[test] #[ignore] fn bild_screen_studio() { let (mut app, tmp) = screen_app(); foto(&mut app, &[], "/tmp/vc-screen.png"); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] fn bildschirm_studio_zeichnet_tippt_speichert_und_exportiert_mit_echten_ereignissen() {
        use iced::{keyboard::{Event as K, Modifiers}, mouse, Event, Point}; use screenstudio::SAkt; use vc_ide_model::screendoc::Werkzeug;
        let (mut app, tmp) = screen_app(); let rel = "graphics/title.vcscreen";
        assert!(app.screens.contains_key(rel) && app.editoren.len() == 1 && app.name().as_deref() == Some(rel), "Textbildschirm öffnet im Studio"); let _ = simuliere(&mut app, &[]);
        let maus = |app: &mut App, ev: Vec<(Event, Point)>| { let _ = simuliere(app, &ev); };
        let klick = |app: &mut App, x: f32, y: f32, taste: mouse::Button| { let p = Point::new(x, y); maus(app, vec![(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(taste)), p), (Event::Mouse(mouse::Event::ButtonReleased(taste)), p)]) };
        let (gx0, gy0, gk) = screen_gitter(&mut app, rel); let (gx, gy) = (move |c: usize| gx0 + c as f32 * gk + gk / 2.0, move |r: usize| gy0 + r as f32 * gk + gk / 2.0);
        let taste = |app: &mut App, z: &str| { let (ev, key, mods) = kuerzel_ev(z); let (msgs, st) = ui_status(app, ev, Point::new(500.0, 300.0)); for m in msgs { let _ = app.update(m); } if st.iter().all(|s| *s == iced::event::Status::Ignored) { let _ = app.update(Msg::Taste(key, mods)); } };
        // Zeichen setzen per Klick (Zeichen 2 vorgewählt), Ziehen, rechte Taste löscht
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::Zeichen(2))); let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::Werkzeug(Werkzeug::Zeichen)));
        let vor = app.screens[rel].doc.code(20, 12); klick(&mut app, gx(20), gy(12), mouse::Button::Left); assert_eq!(app.screens[rel].doc.code(20, 12), 2, "Klick setzt das Zeichen (vorher {vor})"); assert!(app.screens[rel].doc.dirty());
        klick(&mut app, gx(20), gy(12), mouse::Button::Right); assert_eq!(app.screens[rel].doc.code(20, 12), 32, "rechte Taste löscht");
        let (p0, p1) = (Point::new(gx(10), gy(14)), Point::new(gx(13), gy(14))); maus(&mut app, vec![(Event::Mouse(mouse::Event::CursorMoved { position: p0 }), p0), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p0), (Event::Mouse(mouse::Event::CursorMoved { position: p1 }), p1), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p1)]);
        assert!((10..=13).all(|x| app.screens[rel].doc.code(x, 14) == 2), "Ziehen zeichnet eine Linie");
        // Einzeltaste T: Text; Klick in die Zelle, tippen; Esc beendet; Cmd+Z macht rückgängig
        taste(&mut app, "T"); assert_eq!(app.screens[rel].doc.werkzeug, Werkzeug::Text); klick(&mut app, gx(5), gy(20), mouse::Button::Left);
        for z in ["H", "I"] { taste(&mut app, z) } assert_eq!((app.screens[rel].doc.code(5, 20), app.screens[rel].doc.code(6, 20)), (8, 9), "Text: Bildschirmcodes H=8 I=9");
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.screens[rel].doc.code(6, 20), 32, "Cmd+Z nimmt das letzte Zeichen zurück");
        // Speichern per Befehl: Datei auf der Platte enthält die Änderung, Original in examples bleibt unberührt
        let _ = app.befehl(Cmd::Speichern); assert!(!app.screens[rel].doc.dirty()); let gespeichert = std::fs::read_to_string(tmp.join(rel)).unwrap(); assert!(gespeichert.contains("\"chars\""));
        // Werkzeugknopf per Maus, Zoom, Zeichensatz wählen
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::Zoom(1))); assert_eq!(app.screens[rel].zoomansicht.zoom, Some(app.screens[rel].skala.get() as f32 + 1.0).filter(|_| false).or(app.screens[rel].zoomansicht.zoom), "Zoom ist von Hand gesetzt"); assert!(app.screens[rel].zoomansicht.zoom.is_some());
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::Zeichensatz(None))); assert!(app.screens[rel].doc.glyphs.is_none() && app.screens[rel].doc.dirty()); let _ = app.befehl(Cmd::Rueckgaengig); assert!(app.screens[rel].doc.glyphs.is_some());
        // Export-Dialog für dieses Bildschirm-Studio (Knopf per Maus, Fokus folgt dem Klick)
        let _ = simuliere(&mut app, &[]); let (ex, ey) = ort(&mut app, "Exportieren …"); klick(&mut app, ex, ey, mouse::Button::Left);
        assert_eq!(app.export.as_ref().map(|d| d.rel.clone()).as_deref(), Some(rel), "{:?}", app.log.last()); assert!(app.export.as_ref().unwrap().cache.error.is_none(), "{:?}", app.export.as_ref().unwrap().cache.error);
        let _ = (K::ModifiersChanged(Modifiers::empty()), std::fs::remove_dir_all(&tmp));
    }
    #[test] #[ignore] fn bild_tile_studio() { let (mut app, tmp) = screen_app(); app.oeffne("graphics/level_tiles.vctiles"); foto(&mut app, &[], "/tmp/vc-tiles.png"); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] fn kachelsatz_studio_malt_waehlt_loescht_mit_banken_speichert_und_exportiert() {
        use iced::{mouse, Event, Point}; use tilestudio::TAkt;
        let (mut app, tmp) = screen_app(); let rel = "graphics/level_tiles.vctiles"; app.oeffne(rel);
        assert!(app.tilesets.contains_key(rel) && app.name().as_deref() == Some(rel), "Kachelsatz öffnet im Studio"); let _ = simuliere(&mut app, &[]);
        let klick = |app: &mut App, x: f32, y: f32, t: mouse::Button| { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(t)), p), (Event::Mouse(mouse::Event::ButtonReleased(t)), p)]); };
        let akt = |app: &mut App, a: TAkt| { let _ = app.update(Msg::TileAkt(rel.into(), a)); };
        // Kachel „stein“ in der Liste anklicken, Zeichen in der Tafel wählen, Zelle in der Mitte setzen; rechte Taste nimmt auf
        let (lx, ly) = ort(&mut app, "1 *"); klick(&mut app, lx, ly, mouse::Button::Left); assert_eq!(app.tilesets[rel].doc.sel, 1, "Klick in die Liste wählt die Kachel");
        let vor = app.tilesets[rel].doc.ts.tiles[1].cells.clone(); { let (_, yh) = ort(&mut app, "Zeichen 1*"); 'suche: for d in (8..24).step_by(4) { for x in (985..1225).step_by(8) { klick(&mut app, x as f32, yh + d as f32, mouse::Button::Left); if app.tilesets[rel].doc.pinsel.code != 1 { break 'suche } } } } let gewaehlt = app.tilesets[rel].doc.pinsel.code; assert_ne!(gewaehlt, 1, "Zeichen in der Tafel gewählt");
        assert_eq!(app.tilesets[rel].doc.ts.tiles[1].cells[0].code, gewaehlt, "gewählte Zelle wird mit dem Zeichen belegt"); let (gx0, gy0, gk) = tile_gitter(&mut app, rel); klick(&mut app, gx0 + 1.0 * gk + gk / 2.0, gy0 + gk / 2.0, mouse::Button::Left); assert_eq!(app.tilesets[rel].doc.ts.tiles[1].cells[1].code, gewaehlt, "Klick in die Zelle malt");
        klick(&mut app, gx0 + gk / 2.0, gy0 + gk + gk / 2.0, mouse::Button::Right); assert_eq!(app.tilesets[rel].doc.cur, 2); assert_eq!(app.tilesets[rel].doc.pinsel.code, vor[2].code, "rechte Taste nimmt das Zeichen der Zelle auf");
        let _ = app.befehl(Cmd::Rueckgaengig); let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.tilesets[rel].doc.ts.tiles[1].cells, vor, "Cmd+Z macht beide Änderungen rückgängig");
        // Kachel neu, duplizieren, zusammenführen; benutzte löschen mit Rückfrage und Anpassung der Bank
        let n0 = app.tilesets[rel].doc.ts.tiles.len(); akt(&mut app, TAkt::Neu); akt(&mut app, TAkt::Dupl); assert_eq!(app.tilesets[rel].doc.ts.tiles.len(), n0 + 2); akt(&mut app, TAkt::Zusammen); assert_eq!(app.tilesets[rel].doc.ts.tiles.len(), n0 + 1);
        akt(&mut app, TAkt::Kachel(1)); akt(&mut app, TAkt::Loeschen); assert!(app.tilesets[rel].doc.frage.is_some(), "benutzte Kachel: Rückfrage"); akt(&mut app, TAkt::FrageNein); assert_eq!(app.tilesets[rel].doc.ts.tiles.len(), n0 + 1);
        akt(&mut app, TAkt::Loeschen); akt(&mut app, TAkt::FrageJa); assert_eq!(app.tilesets[rel].doc.ts.tiles.len(), n0); assert!(tmp.join("graphics/level.vcscreen").is_file());
        // Name, Farbe, Speichern per Befehl, Export-Dialog
        akt(&mut app, TAkt::Name("fels".into())); akt(&mut app, TAkt::Kachelfarbe(7)); assert!(app.tilesets[rel].doc.dirty()); let _ = app.befehl(Cmd::Speichern); assert!(!app.tilesets[rel].doc.dirty()); assert!(std::fs::read_to_string(tmp.join(rel)).unwrap().contains("fels"));
        let _ = simuliere(&mut app, &[]); let (ex, ey) = ort(&mut app, "Exportieren …"); klick(&mut app, ex, ey, mouse::Button::Left); assert_eq!(app.export.as_ref().map(|d| d.rel.clone()).as_deref(), Some(rel), "{:?}", app.log.last()); assert!(app.export.as_ref().unwrap().cache.error.is_none(), "{:?}", app.export.as_ref().unwrap().cache.error);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_bank_studio() { let (mut app, tmp) = screen_app(); app.oeffne("graphics/level.vcscreen"); foto(&mut app, &[], "/tmp/vc-bank.png"); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] fn bank_studio_malt_kacheln_zeichen_screens_auswahl_kachel_speichert_und_exportiert() {
        use iced::{mouse, Event, Point}; use bankstudio::BAkt; use vc_ide_model::bankdoc::{Modus, Werkzeug};
        let (mut app, tmp) = screen_app(); let rel = "graphics/level.vcscreen"; app.oeffne(rel);
        assert!(app.banken.contains_key(rel) && app.name().as_deref() == Some(rel), "Bank öffnet im Studio"); let _ = simuliere(&mut app, &[]);
        let klick = |app: &mut App, x: f32, y: f32, t: mouse::Button| { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(t)), p), (Event::Mouse(mouse::Event::ButtonReleased(t)), p)]); };
        let akt = |app: &mut App, a: BAkt| { let _ = app.update(Msg::BankAkt(rel.into(), a)); };
        let taste = |app: &mut App, z: &str| { let (ev, key, mods) = kuerzel_ev(z); let (msgs, st) = ui_status(app, ev, Point::new(500.0, 300.0)); for m in msgs { let _ = app.update(m); } if st.iter().all(|s| *s == iced::event::Status::Ignored) { let _ = app.update(Msg::Taste(key, mods)); } };
        let gitter = |app: &App| app.banken[rel].doc.bank.screens[0].grid.clone();
        // Kachel in der Palette wählen, in den Screen malen; rechte Taste nimmt die Kachel auf
        let (lx, ly) = ort(&mut app, "Kachel 0*"); 'pal: for dy in (0..140).step_by(8) { for dx in (-100..140).step_by(8) { klick(&mut app, lx + dx as f32, ly - 12.0 - dy as f32, mouse::Button::Left); if app.banken[rel].doc.pinsel == 1 { break 'pal } } } assert_eq!(app.banken[rel].doc.pinsel, 1, "Klick in die Kachelpalette");
        let (bx0, by0, bk) = bank_gitter(&mut app, rel); let tw = app.banken[rel].doc.ts.as_ref().unwrap().tw as f32; let kachel_mitte = move |tx: usize, ty: usize| (bx0 + (tx as f32 * tw + tw / 2.0) * bk, by0 + (ty as f32 * tw + tw / 2.0) * bk); let zelle_mitte = move |cx: usize, cy: usize| (bx0 + (cx as f32 + 0.5) * bk, by0 + (cy as f32 + 0.5) * bk);
        assert_eq!(gitter(&app)[3][5], 0); { let (x, y) = kachel_mitte(5, 3); klick(&mut app, x, y, mouse::Button::Left); } assert_eq!(gitter(&app)[3][5], 1, "Klick malt die Kachel"); assert!(app.banken[rel].doc.dirty());
        { let (x, y) = kachel_mitte(0, 10); klick(&mut app, x, y, mouse::Button::Right); } assert_eq!(app.banken[rel].doc.pinsel, gitter(&app)[10][0], "rechte Taste nimmt die Kachel unter dem Zeiger auf");
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(gitter(&app)[3][5], 0, "Cmd+Z macht das Malen rückgängig");
        // Zeichenmodus per Tab, Text tippen, Patches
        let tab = |app: &mut App| { let _ = app.update(Msg::Taste(iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab), iced::keyboard::Modifiers::empty())); }; tab(&mut app); assert_eq!(app.banken[rel].doc.modus, Modus::Zelle); akt(&mut app, BAkt::Werkzeug(Werkzeug::Text)); { let (x, y) = zelle_mitte(3, 2); klick(&mut app, x, y, mouse::Button::Left); } taste(&mut app, "H"); taste(&mut app, "I");
        assert_eq!(app.banken[rel].doc.bank.screens[0].patches.iter().filter(|p| p.y == 2 && (p.x == 3 || p.x == 4)).count(), 2, "Text setzt Patches ab der angeklickten Zelle");
        // Auswahl → Kachel anlegen: Rückfrage, anlegen, Screen verweist darauf
        tab(&mut app); akt(&mut app, BAkt::Werkzeug(Werkzeug::Auswahl)); app.banken.get_mut(rel).unwrap().doc.auswahl = Some(((0, 10), (0, 10))); let n0 = app.banken[rel].doc.ts.as_ref().unwrap().tiles.len();
        akt(&mut app, BAkt::AlsKachel); assert!(app.banken[rel].doc.neue_kachel.is_some(), "Rückfrage für die neue Kachel"); let _ = simuliere(&mut app, &[]); akt(&mut app, BAkt::NKName("boden".into())); akt(&mut app, BAkt::NKOk);
        let d = &app.banken[rel].doc; assert!(d.neue_kachel.is_none() && d.ts.as_ref().unwrap().tiles.len() <= n0 + 1 && d.ts_dirty == (d.ts.as_ref().unwrap().tiles.len() > n0), "Kachel angelegt oder Duplikat benutzt");
        // Screens: neu, duplizieren, verschieben, löschen, umbenennen
        akt(&mut app, BAkt::ScreenNeu); akt(&mut app, BAkt::ScreenDupl); akt(&mut app, BAkt::ScreenName("zwei".into())); assert_eq!(app.banken[rel].doc.bank.screens.len(), 3); assert_eq!(app.banken[rel].doc.bank.screens[2].name, "zwei"); akt(&mut app, BAkt::ScreenLoeschen); assert_eq!(app.banken[rel].doc.bank.screens.len(), 2); akt(&mut app, BAkt::Screen(0));
        // Größe, Farbe
        akt(&mut app, BAkt::Groesse(40, 24, 1, 1, true)); assert_eq!(app.banken[rel].doc.bank.screens[0].origin, (1, 1)); akt(&mut app, BAkt::Farbe("bg", 0)); assert_eq!(app.banken[rel].doc.bank.screens[0].colors["bg"], 0);
        // Speichern per Befehl: Bank und Satz auf der Platte; Export-Knopf
        let _ = app.befehl(Cmd::Speichern); assert!(!app.banken[rel].doc.dirty()); let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.join(rel)).unwrap()).unwrap(); assert_eq!(v["screens"][0]["tiles"]["origin"][0], 1);
        let _ = simuliere(&mut app, &[]); let (ex, ey) = ort(&mut app, "Exportieren …"); klick(&mut app, ex, ey, mouse::Button::Left); assert_eq!(app.export.as_ref().map(|d| d.rel.clone()).as_deref(), Some(rel), "{:?}", app.log.last()); assert!(app.export.as_ref().unwrap().cache.error.is_none(), "{:?}", app.export.as_ref().unwrap().cache.error);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn bank_per_dateipfad_oeffnen_kachelsatz_aendern_speichern_und_exportieren_findet_den_kachelsatz() {
        use bankstudio::BAkt;
        let tmp = kopie_von_beispiel("examples/c64/gfx_demo"); einstellungen::test_konfig(tmp.join("konf"));
        let z = project::ziel_fuer(&tmp.join("graphics/level.vcscreen")).expect("Ziel"); assert_eq!(z.root, tmp, "Wurzel = Ordner mit der .vcprj");
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(z.root.clone())); let rel = z.datei.clone().unwrap(); app.oeffne(&rel);
        assert!(app.banken.contains_key(&rel), "Bank offen");
        let akt = |app: &mut App, a: BAkt| { let _ = app.update(Msg::BankAkt(rel.clone(), a)); };
        // Kachelsatz ändern: neue Kachel aus Auswahl, dann Speichern und Export (Vorschau und echter Export)
        { let d = &mut app.banken.get_mut(&rel).unwrap().doc; d.auswahl = Some(((0, 10), (0, 10))); }
        akt(&mut app, BAkt::Werkzeug(vc_ide_model::bankdoc::Werkzeug::Auswahl)); akt(&mut app, BAkt::AlsKachel); akt(&mut app, BAkt::NKName("boden2".into())); akt(&mut app, BAkt::NKOk);
        let _ = app.befehl(Cmd::Speichern); assert!(!app.banken[&rel].doc.dirty(), "{:?}", app.log.last());
        assert!(tmp.join("graphics/level_tiles.vctiles").exists());
        let _ = app.befehl(Cmd::Exportieren); let d = app.export.as_ref().expect("Export-Dialog"); assert!(d.cache.error.is_none(), "Vorschau: {:?}", d.cache.error);
        let _ = app.export_ausfuehren(exportdlg::ExportAus::Speichern); assert!(tmp.join("build/export").is_dir(), "{:?}", app.log.last()); assert!(!app.log.iter().any(|l| l.contains("No such file")), "{:?}", app.log);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn bank_studio_gitter_und_kachelgrenzen_stehen_ruhig_ohne_mausbewegung() {
        let (mut app, tmp) = screen_app(); let rel = "graphics/level.vcscreen"; app.oeffne(rel); let _ = simuliere(&mut app, &[]);
        let dir = std::env::temp_dir().join(format!("vc_gitter_{}", std::process::id())); let _ = std::fs::create_dir_all(&dir);
        let mut aufnahme = |app: &mut App, name: &str| -> Vec<u8> { let f = dir.join(name); foto(app, &[], f.to_str().unwrap()); tiny_skia::Pixmap::load_png(&f).unwrap().data().to_vec() };
        let setze = |app: &mut App, kg: bool, zg: bool| { let s = app.banken.get_mut(rel).unwrap(); s.kachelgrenzen = kg; s.zellgitter = zg; };
        setze(&mut app, false, false); let aus = aufnahme(&mut app, "aus.png");
        setze(&mut app, true, false); let kachel = aufnahme(&mut app, "kachel.png"); setze(&mut app, true, false); let kachel2 = aufnahme(&mut app, "kachel2.png");
        setze(&mut app, false, true); let zelle = aufnahme(&mut app, "zelle.png");
        assert_ne!(aus, kachel, "Kachelgrenzen sind ohne Mausbewegung sichtbar"); assert_ne!(aus, zelle, "Zellgitter ist ohne Mausbewegung sichtbar"); assert_eq!(kachel, kachel2, "zwei Bilder ohne Eingabe sind gleich (kein Flackern)");
        let _ = std::fs::remove_dir_all(&dir); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn jedes_geoeffnete_projekt_kommt_in_die_zuletzt_liste_auch_per_startargument_und_dateipfad() {
        let tmp = kopie_von_beispiel("examples/c64/gfx_demo"); let liste = tmp.join("zuletzt.json");
        let (mut app, _) = App::neu(); vc_ide_model::recent::umleiten(Some(liste.clone())); app.einst = Einstellungen::default();
        // Öffnen über eine Datei in einem anderen Projekt (wie Startargument/Finder „Öffnen mit“)
        assert!(app.projekt_oeffnen(tmp.join("graphics/level.vcscreen")));
        let r = vc_ide_model::recent::load(Some(&app.data)); let ok = r.iter().any(|e| e.path == tmp || std::fs::canonicalize(&e.path).ok() == std::fs::canonicalize(&tmp).ok());
        let text = std::fs::read_to_string(&liste).unwrap_or_default();
        vc_ide_model::recent::umleiten(None);
        assert!(ok, "Projekt steht in der Zuletzt-Liste: {text}"); assert!(app.start.recent.iter().any(|e| e.path == tmp || std::fs::canonicalize(&e.path).ok() == std::fs::canonicalize(&tmp).ok()), "Startseite zeigt es");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn textbildschirm_in_kacheln_zerlegen_erzeugt_satz_und_bank_und_oeffnet_sie() {
        let (mut app, tmp) = screen_app(); let rel = "graphics/title.vcscreen"; assert!(app.screens.contains_key(rel));
        let vor = std::fs::read(tmp.join(rel)).unwrap(); let _ = app.update(Msg::ScreenAkt(rel.into(), screenstudio::SAkt::Zerlegen(2, 2)));
        assert!(tmp.join("graphics/title_tiles.vctiles").is_file() && tmp.join("graphics/title_bank.vcscreen").is_file(), "{:?}", app.log.last()); assert_eq!(std::fs::read(tmp.join(rel)).unwrap(), vor, "der Bildschirm selbst bleibt unverändert");
        assert!(app.banken.contains_key("graphics/title_bank.vcscreen") && app.name().as_deref() == Some("graphics/title_bank.vcscreen"), "die neue Bank öffnet im Studio"); assert!(app.log.iter().any(|l| l.starts_with("In Kacheln zerlegt (2×2)")), "{:?}", app.log);
        // zweites Mal: vorhandene Dateien bleiben unberührt, Meldung statt Überschreiben
        let _ = app.update(Msg::ScreenAkt(rel.into(), screenstudio::SAkt::Zerlegen(2, 2))); assert!(app.log.last().is_some_and(|l| l.starts_with("Zerlegen nicht möglich") && l.contains("gibt es schon")), "{:?}", app.log.last());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn bildschirm_studio_im_muster_zerlegen_dialog_mit_vorschau_einpassen_zoom_und_esc_bricht_den_rahmen_ab() {
        use iced::{mouse, Event, Point}; use screenstudio::SAkt; use vc_ide_model::screendoc::Werkzeug;
        let (mut app, tmp) = screen_app(); let rel = "graphics/title.vcscreen"; let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let t = texte(&mut app); for n in ["SCR", "title.vcscreen", "Zerlegen …", "Exportieren …", "Speichern", "Werkzeug", "Dokument", "Info", "Einpassen"] { assert!(t.iter().any(|x| x == n), "{n} sichtbar: {t:?}") }
        assert!(!t.iter().any(|x| x == "Zerlegen 2×2"), "die Knopfreihe Zerlegen 2×2/3×3/4×4 ist weg");
        // Einpassen: ganzzahlig, kein Balken; Zoom von Hand in ganzen Stufen, Einpassen setzt zurück
        assert!(app.screens[rel].zoomansicht.zoom.is_none()); let sk0 = app.screens[rel].skala.get(); assert!(sk0 >= 1);
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::Zoom(1))); assert_eq!(app.screens[rel].zoomansicht.zoom, Some(sk0 as f32 + 1.0).filter(|z| *z <= 8.0).or(Some(8.0)), "ganze Stufe größer (höchstens 8)");
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::Einpassen)); assert!(app.screens[rel].zoomansicht.zoom.is_none());
        // Zerlegen-Dialog mit Vorschau: öffnet, zeigt „n Kacheln …“, Größe wählbar, Abbrechen schreibt nichts, Zerlegen legt Satz und Bank an
        let vor = std::fs::read(tmp.join(rel)).unwrap();
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::ZerlegenDialog(Some((2, 2))))); let t = texte(&mut app); assert!(t.iter().any(|x| x == "In Kacheln zerlegen") && t.iter().any(|x| x.contains("Kacheln,") && x.contains("Byte")), "Vorschau: {t:?}");
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::ZerlegenDialog(Some((3, 3))))); assert_eq!(app.screens[rel].zerlegen, Some((3, 3)));
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::ZerlegenDialog(None))); assert!(app.screens[rel].zerlegen.is_none()); assert!(!tmp.join("graphics/title_tiles.vctiles").exists(), "Abbrechen schreibt nichts");
        let _ = app.update(Msg::ScreenAkt(rel.into(), SAkt::ZerlegenDialog(Some((2, 2))))); let (zx, zy) = ort(&mut app, "Zerlegen"); let p = Point::new(zx, zy);
        let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]);
        assert!(tmp.join("graphics/title_tiles.vctiles").is_file(), "{:?}", app.log.last()); assert_eq!(std::fs::read(tmp.join(rel)).unwrap(), vor, "Bildschirm bleibt unverändert");
        // Rahmen: Esc bricht den begonnenen Rahmen ab (Ziehvorschau verschwindet), nichts wird gesetzt
        let rel2 = "graphics/title.vcscreen"; app.oeffne(rel2); if let Some(s) = app.screens.get_mut(rel2) { s.doc.werkzeug = Werkzeug::Rahmen; s.doc.von = Some((3, 3)) }
        let verarbeitet = app.screen_taste(rel2, &iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape), iced::keyboard::Modifiers::empty()); assert!(verarbeitet, "Esc wird vom Bildschirm-Studio verarbeitet");
        if let Some(s) = app.screens.get(rel2) { assert!(s.doc.von.is_none(), "Esc bricht den Rahmen ab") }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn kachelsatz_studio_im_muster_karten_fuellstand_eigenschaften_einpassen_und_zoom() {
        use tilestudio::TAkt;
        let (mut app, tmp) = screen_app(); let rel = "graphics/level_tiles.vctiles"; app.oeffne(rel); let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
        let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect();
        for n in ["MAP", "level_tiles.vctiles", "Exportieren …", "Speichern", "Füllstand", "Werkzeug", "Eigenschaften", "Kachel", "Info", "Einpassen", "Kacheln · 4"] { assert!(t.iter().any(|x| x == n), "{n} sichtbar: {t:?}") }
        assert!(t.iter().any(|x| x.contains("von 256 Zeichen belegt")) && t.iter().any(|x| x.contains("Kacheln") && x.starts_with("4 von")), "Füllstand der Zeichen und der Kacheln: {t:?}");
        assert!(t.iter().any(|x| x == "eigene Marken, die dein Programm abfragen kann"), "Erklärung der Eigenschaften"); assert!(!t.iter().any(|x| x == "Größe für den ganzen Satz" && false));
        // Einpassen in ganzen Stufen, Zoom von Hand, Einpassen setzt zurück
        assert!(app.tilesets[rel].zoomansicht.zoom.is_none()); let sk = app.tilesets[rel].skala.get(); assert!(sk >= 1);
        let _ = app.update(Msg::TileAkt(rel.into(), TAkt::Zoom(-1))); assert_eq!(app.tilesets[rel].zoomansicht.zoom, Some((sk as f32 - 1.0).max(1.0)), "ganze Stufe kleiner");
        let _ = app.update(Msg::TileAkt(rel.into(), TAkt::Einpassen)); assert!(app.tilesets[rel].zoomansicht.zoom.is_none() && app.tilesets[rel].zoomansicht.pan == (0.0, 0.0));
        // Karten ein-/ausklappen (Zustand im Modell)
        let _ = app.update(Msg::TileAkt(rel.into(), TAkt::Karte("eigenschaften"))); assert!(app.tilesets[rel].karten_zu.contains("eigenschaften")); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect(); assert!(!t.iter().any(|x| x == "solid"), "zugeklappt: Inhalt weg");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn bank_studio_im_muster_karten_liste_einpassen_zoom_und_esc_bricht_linie_ab() {
        use bankstudio::BAkt; use vc_ide_model::bankdoc::Werkzeug;
        let (mut app, tmp) = screen_app(); let rel = "graphics/level.vcscreen"; app.oeffne(rel); let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let _ = texte(&mut app); assert_eq!(app.banken[rel].minis.borrow().1.len(), app.banken[rel].doc.bank.screens.len(), "Miniatur je Screen"); let st0 = app.banken[rel].minis.borrow().0; { let d = &mut app.banken.get_mut(rel).unwrap().doc; d.cur = 0; } let _ = texte(&mut app); assert_eq!(app.banken[rel].minis.borrow().0, st0, "Inhalt unverändert: Miniaturen werden nicht neu gerechnet");
        let t = texte(&mut app); for n in ["BNK", "level.vcscreen", "Exportieren …", "Speichern", "Werkzeug", "Dokument", "Info", "Einpassen", "Screens · 1", "Kacheln", "Zeichen"] { assert!(t.iter().any(|x| x == n), "{n} sichtbar: {t:?}") }
        let _ = app.update(Msg::BankAkt(rel.into(), BAkt::Mehr)); let t = texte(&mut app); assert!(t.iter().any(|x| x == "Stift"), "Werkzeuge im ⋯-Menü (Studio unter 1180 px): {t:?}"); let _ = app.update(Msg::BankAkt(rel.into(), BAkt::Mehr));
        assert!(!t.iter().any(|x| x == "Duplizieren"), "die dritte Leistenreihe (Screen-Reiter) ist weg: Screens stehen in der Liste");
        // Einpassen in ganzen Stufen, Zoom von Hand, Einpassen setzt zurück
        assert!(app.banken[rel].zoomansicht.zoom.is_none()); let sk = app.banken[rel].skala.get(); assert!(sk >= 1);
        let _ = app.update(Msg::BankAkt(rel.into(), BAkt::Zoom(1))); assert!(app.banken[rel].zoomansicht.zoom.is_some_and(|z| z == (sk as f32 + 1.0).min(8.0)), "ganze Stufe größer");
        let _ = app.update(Msg::BankAkt(rel.into(), BAkt::Einpassen)); assert!(app.banken[rel].zoomansicht.zoom.is_none());
        // Esc bricht eine begonnene Linie ab (Ziehvorschau verschwindet)
        { let s = app.banken.get_mut(rel).unwrap(); s.doc.werkzeug_waehlen(Werkzeug::Linie); s.doc.von = Some((2, 2)); }
        let verarbeitet = app.bank_taste(rel, &iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape), iced::keyboard::Modifiers::empty()); assert!(verarbeitet && app.banken[rel].doc.von.is_none(), "Esc bricht die Linie ab");
        // Karten einklappen (Zustand im Modell)
        let _ = app.update(Msg::BankAkt(rel.into(), BAkt::Karte("werkzeug"))); assert!(app.banken[rel].karten_zu.contains("werkzeug"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn disk_studio_im_muster_kopf_leiste_karten_kette_block_n_von_m_und_interleave_hinweis() {
        use diskstudio::DAkt;
        let (mut app, tmp) = disk_app(); let rel = "spiel.d64"; let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let t = texte(&mut app); for n in ["D64", "spiel.d64", "Datei hinzufügen …", "Speichern unter …", "Speichern", "Validieren", "Zustand", "Sektor", "In das Projekt übernehmen"] { assert!(t.iter().any(|x| x == n), "{n} sichtbar: {t:?}") }
        assert!(t.iter().any(|x| x.starts_with("Interleave ") && x.contains("bewusst auseinander")), "Interleave-Hinweis: {t:?}");
        assert!(t.iter().any(|x| x.starts_with("Datei im Verzeichnis wählen")), "ohne Datei: Hinweis auf die Kettenanzeige");
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Datei(2))); let t = texte(&mut app); assert!(t.iter().any(|x| x.starts_with("Block 1 von ") && x.contains("der Datei")), "Kette: {t:?}");
        let (n, m) = app.disks[rel].kettenposition().expect("Position"); assert_eq!(n, 1); assert!(m >= 2, "DATEN (700 Bytes) hat mehrere Blöcke: {m}");
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Folgen)); let t = texte(&mut app); assert!(t.iter().any(|x| x.starts_with(&format!("Block 2 von {m}"))), "{t:?}");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    fn song_app() -> (App, std::path::PathBuf) {
        let tmp = kopie_von_beispiel("examples/c64/music_demo"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone())); app.oeffne("music/tune.vcsong"); (app, tmp)
    }
    #[test] #[ignore] fn bild_song_studio() { let (mut app, tmp) = song_app(); foto(&mut app, &[], "/tmp/vc-song.png"); let _ = app.befehl(Cmd::TrackerInstrumente); foto(&mut app, &[], "/tmp/vc-song-ins.png"); let _ = app.befehl(Cmd::TrackerAblauf); foto(&mut app, &[], "/tmp/vc-song-ablauf.png"); let _ = std::fs::remove_dir_all(&tmp); }
    /// Ziehen aus dem Projektbaum in den Quelltext: Einfügemarke unter dem Zeiger, Loslassen setzt die Include-Zeile; Ablehnungen als Hinweis
    #[test] fn ziehen_aus_dem_baum_in_den_quelltext_setzt_die_include_zeile() {
        use iced::{mouse, Event, Point};
        let tmp = std::env::temp_dir().join(format!("vcgui-drop-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(tmp.join("lib")).unwrap();
        std::fs::write(tmp.join("a.asm"), ".target c64\n.org $0801\nstart: lda #$20\n  jmp start\n").unwrap(); std::fs::write(tmp.join("lib/b.inc"), "x = 1\n").unwrap(); std::fs::write(tmp.join("lib/d.bin"), [1u8, 2, 3]).unwrap(); std::fs::write(tmp.join("n.bas"), "10 end\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.asm");
        let cw = 7.8_f32; let (x0, y0, lh) = (330.0_f32, 94.0_f32, 19.0_f32); let ort = |zeile: usize, spalte: f32| Point::new(x0 + (spalte + 0.5) * cw, y0 + zeile as f32 * lh + 3.0);
        let bewege = |app: &mut App, p: Point| simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]);
        let loslassen = |app: &mut App, p: Point| simuliere(app, &[(Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]);
        // Ziehen beginnt (Datei, kein Ordner); Zeiger über dem Editor: Einfügemarke vor der Zeile unter dem Zeiger
        let _ = app.update(Msg::BaumZugStart("lib/b.inc".into(), false));
        // Zeiger senkrecht über die Zeilen führen: die Marke wandert nie rückwärts und steht abwechselnd vor und hinter Zeilen
        let mut spur: Vec<(f32, usize)> = vec![]; for dy in 0..120 { let q = Point::new(x0 + 40.0, y0 - 6.0 + dy as f32); bewege(&mut app, q); if let Some(m) = app.ed().unwrap().einfuegemarke { spur.push((q.y, m)) } }
        let ende = spur.windows(2).position(|w| w[0].1 > w[1].1).unwrap_or(spur.len()); spur.truncate(ende + 1); // hinter dem Textende hält die Marke an der letzten Zeile
        assert!(spur.windows(2).all(|w| w[0].1 <= w[1].1), "{spur:?}"); let ms: std::collections::BTreeSet<usize> = spur.iter().map(|x| x.1).collect(); assert!(ms.len() >= 4, "{spur:?}");
        let (ya, mv) = *spur.iter().find(|x| x.1 == 2).expect("Marke 2"); let _ = mv; let p = Point::new(x0 + 40.0, ya);
        bewege(&mut app, p); assert_eq!(app.ed().unwrap().einfuegemarke, Some(2), "Marke vor Zeile 2");
        bewege(&mut app, Point::new(5.0, 5.0)); assert_eq!(app.ed().unwrap().einfuegemarke, None, "Zeiger draußen: Marke weg");
        bewege(&mut app, p); loslassen(&mut app, p);
        let z = app.ed().unwrap().lines.clone(); assert_eq!(z[2], ".include \"lib/b.inc\"", "{z:?}"); assert!(app.baum_zug.is_none() && app.ed().unwrap().einfuegemarke.is_none());
        assert!(app.hinweis.starts_with("Eingebunden: .include"), "{:?}", app.hinweis);
        // schon eingebunden: nichts eingefügt, Hinweis
        let vorher = app.ed().unwrap().text(); let _ = app.update(Msg::BaumZugStart("lib/b.inc".into(), false)); bewege(&mut app, p); loslassen(&mut app, p);
        assert_eq!(app.ed().unwrap().text(), vorher, "nicht doppelt"); assert!(app.hinweis.contains("schon eingebunden"), "{:?}", app.hinweis);
        // Binärdatei: .incbin; Ordner und sich selbst: Hinweis ohne Änderung
        let _ = app.update(Msg::BaumZugStart("lib/d.bin".into(), false)); let q = ort(0, 3.0); bewege(&mut app, q); loslassen(&mut app, q); let z = app.ed().unwrap().lines.clone(); assert!(z.iter().any(|l| l == ".incbin \"lib/d.bin\""), "{z:?}");
        let vorher = app.ed().unwrap().text(); let _ = app.update(Msg::BaumZugStart("a.asm".into(), false)); bewege(&mut app, q); loslassen(&mut app, q); assert_eq!(app.ed().unwrap().text(), vorher); assert!(!app.hinweis.is_empty());
        // Ordner lassen sich nicht ablegen (Editor meldet gar nichts: kein Ziehen-Modus)
        let _ = app.update(Msg::BaumZugStart("lib".into(), true)); bewege(&mut app, q); assert_eq!(app.ed().unwrap().einfuegemarke, None);
        let _ = app.update(Msg::BaumZugAbbruch);
        // Ohne Maus: Rechtsklick ▸ „Als Include einfügen“ setzt die Zeile unter den Cursor; schon eingebunden: Sprung zur vorhandenen Zeile
        std::fs::write(tmp.join("lib/c.inc"), "y = 2\n").unwrap(); app.projekt_neu_laden();
        app.ed().unwrap().set_cursor(Pos { line: 0, col: 2 }, false); app.ziel = Some("lib/c.inc".into()); let _ = app.befehl(Cmd::AlsInclude);
        let z = app.ed().unwrap().lines.clone(); assert_eq!(z[1], ".include \"lib/c.inc\"", "{z:?}");
        app.ed().unwrap().set_cursor(Pos { line: 5, col: 0 }, false); app.ziel = Some("lib/c.inc".into()); let _ = app.befehl(Cmd::AlsInclude);
        assert_eq!(app.ed().unwrap().cursor.line, 1, "Sprung zur vorhandenen Zeile"); assert!(app.hinweis.contains("schon eingebunden (Zeile 2)"), "{:?}", app.hinweis);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Menüleiste im Fenster (Windows, Linux): Inhalt aus der Befehlstabelle, Öffnen/Schließen/Auslösen, Tastatur, Zen, Einstellung, schmales Fenster
    #[test] fn menueleiste_im_fenster_zeigt_die_befehlstabelle_und_bedient_sich_mit_maus_und_tastatur() {
        use iced::keyboard::{key::Named, Key, Modifiers};
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let (mut app, tmp) = debug_app(); app.dbg = None; einstellungen::test_konfig(tmp.join("konf"));
        // unter macOS ohne Test-Schalter keine Leiste im Fenster (System-Menüleiste); sonst nach Einstellung
        assert_eq!(app.menue_sichtbar(), !cfg!(target_os = "macos"), "Vorgabe: nur wo es keine System-Menüleiste gibt");
        app.menue_test = true; assert!(app.menue_sichtbar());
        // Menütitel und Einträge kommen aus MENUES/BEFEHLE (keine zweite Liste)
        // die Titel sind Rich-Text (unterstrichener Buchstabe) und erscheinen nicht als einfacher Text; breit: kein „Menü“-Knopf
        let t = texte(&mut app); assert!(!t.iter().any(|x| x.contains("☰ Menü")), "breit: einzelne Titel, kein Sammelknopf");
        // jedes Menü (und jedes Untermenü) zeigt genau seine Befehle aus der Tabelle: Namen, Kürzel und Anzahl; Untermenüs sind über eine eigene Zeile erreichbar
        for m in befehle::MENUES {
            let mut gesehen = 0usize;
            for u in std::iter::once("").chain(befehle::untermenues(m).iter().copied()) {
                let ziel = if u.is_empty() { m.to_string() } else { format!("{m}/{u}") };
                let _ = app.update(Msg::KontextZu); let _ = app.update(Msg::MenueAuf(ziel.clone())); let k = app.kontext.as_ref().expect("Menü offen"); assert_eq!((k.art, k.ziel.as_deref()), (Art::Menue, Some(ziel.as_str())));
                let (zeilen, cmds) = app.kontext_zeilen(k); let erwartet: Vec<&befehle::Info> = befehle::menue_eintraege(m, u).collect();
                let befehlszeilen: Vec<(&Zeile<Msg>, Cmd)> = zeilen.iter().zip(&cmds).filter_map(|(z, c)| c.map(|c| (z, c))).collect();
                assert_eq!(befehlszeilen.len(), erwartet.len(), "{ziel}: gleich viele Einträge wie die Befehlstabelle");
                for ((z, c), i) in befehlszeilen.iter().zip(&erwartet) { assert_eq!(*c, i.cmd); assert!(z.name.starts_with(befehle::menuename(i)), "{ziel}: {} vs {}", z.name, i.name); assert_eq!(z.kuerzel, kontext::kuerzel(i.cmd, cfg!(target_os = "macos")), "{ziel}/{}: Kürzel", i.name) }
                gesehen += erwartet.len();
                if u.is_empty() { let subs = zeilen.iter().filter(|z| z.aktiv && z.name.ends_with(" ▸")).count(); assert_eq!(subs, befehle::untermenues(m).len(), "{m}: eine Zeile je Untermenü") } else { assert!(zeilen[0].name.starts_with('◂'), "{ziel}: erste Zeile führt zurück") }
            }
            assert_eq!(gesehen, befehle::BEFEHLE.iter().filter(|i| i.menue == *m).count(), "{m}: jeder Befehl genau einmal erreichbar");
        }
        // Tastatur: Pfeil rechts öffnet ein Untermenü, Pfeil links schließt es; Enter auf der Zeile öffnet es ebenfalls
        { use iced::keyboard::{key::Named, Key, Modifiers}; let _ = app.update(Msg::KontextZu); app.menue_oeffnen("Bearbeiten"); let (z, _) = app.kontext_zeilen(app.kontext.as_ref().unwrap()); let i = z.iter().position(|x| x.name == "Markdown ▸").expect("Untermenü Markdown"); app.kontext.as_mut().unwrap().sel = i;
            let _ = app.update(Msg::Taste(Key::Named(Named::ArrowRight), Modifiers::empty())); assert_eq!(app.kontext.as_ref().and_then(|k| k.ziel.clone()).as_deref(), Some("Bearbeiten/Markdown"), "rechts öffnet das Untermenü");
            let _ = app.update(Msg::Taste(Key::Named(Named::ArrowLeft), Modifiers::empty())); assert_eq!(app.kontext.as_ref().and_then(|k| k.ziel.clone()).as_deref(), Some("Bearbeiten"), "links schließt es wieder");
            app.kontext.as_mut().unwrap().sel = i; let _ = app.update(Msg::Taste(Key::Named(Named::Enter), Modifiers::empty())); assert_eq!(app.kontext.as_ref().and_then(|k| k.ziel.clone()).as_deref(), Some("Bearbeiten/Markdown"), "Enter öffnet das Untermenü");
            let _ = app.update(Msg::KontextZu); }
        // gleicher Titel schließt wieder; Esc schließt
        let _ = app.update(Msg::MenueAuf("Datei".into())); let _ = app.update(Msg::MenueAuf("Datei".into())); assert!(app.kontext.is_none(), "zweiter Klick schließt");
        let _ = app.update(Msg::MenueAuf("Ansicht".into())); let _ = app.update(Msg::Taste(Key::Named(Named::Escape), Modifiers::empty())); assert!(app.kontext.is_none(), "Esc schließt");
        app.kuerzel_windows = true; // Windows/Linux-Pfade (Alt+Buchstabe); unter macOS gibt es sie nicht
        // Mnemonics: je Menü ein Buchstabe aus dem Namen, eindeutig, mit keinem Befehlskürzel belegt; AltGr (Strg+Alt) und Alt+Umschalt öffnen nichts
        let bs: Vec<char> = befehle::MENUES.iter().map(|m| menue_buchstabe(m).expect("jedes Menü hat einen Buchstaben")).collect();
        for (m, b) in befehle::MENUES.iter().zip(&bs) { assert!(m.to_lowercase().contains(*b), "{m}: Buchstabe {b} steht im Namen (Unterstreichung)"); assert_eq!(bs.iter().filter(|x| *x == b).count(), 1, "Buchstabe {b} eindeutig"); assert!(!befehle::BEFEHLE.iter().any(|i| i.tasten.eq_ignore_ascii_case(&format!("Alt+{b}"))), "Alt+{b} schon als Kürzel belegt") }
        let _ = app.update(Msg::Taste(Key::Character("d".into()), Modifiers::CTRL | Modifiers::ALT)); assert!(app.kontext.is_none(), "AltGr+D öffnet kein Menü");
        let _ = app.update(Msg::Taste(Key::Character("d".into()), Modifiers::ALT | Modifiers::SHIFT)); assert!(app.kontext.is_none(), "Alt+Umschalt+D öffnet kein Menü");
        // Alt+Buchstabe öffnet, links/rechts wechselt, Enter löst aus (Ansicht ▸ Hell/Dunkel umschalten ändert das Farbschema)
        let _ = app.update(Msg::Taste(Key::Character("d".into()), Modifiers::ALT)); assert_eq!(app.kontext.as_ref().and_then(|k| k.ziel.clone()).as_deref(), Some("Datei"), "Alt+D");
        let _ = app.update(Msg::Taste(Key::Named(Named::ArrowRight), Modifiers::empty())); assert_eq!(app.kontext.as_ref().and_then(|k| k.ziel.clone()).as_deref(), Some("Bearbeiten"), "rechts: nächstes Menü");
        let _ = app.update(Msg::Taste(Key::Named(Named::ArrowLeft), Modifiers::empty())); let _ = app.update(Msg::Taste(Key::Named(Named::ArrowLeft), Modifiers::empty())); assert_eq!(app.kontext.as_ref().and_then(|k| k.ziel.clone()).as_deref(), Some("Hilfe"), "links über den Anfang: letztes Menü");
        for (c, n) in [("a", "Ansicht"), ("p", "Projekt"), ("h", "Hilfe"), ("b", "Bearbeiten")] { let _ = app.update(Msg::Taste(Key::Character(c.into()), Modifiers::ALT)); assert_eq!(app.kontext.as_ref().and_then(|k| k.ziel.clone()).as_deref(), Some(n), "Alt+{c}") }
        // macOS: Alt+Buchstabe öffnet kein Menü, Option+Taste bleibt Text, keine Unterstreichung
        app.kuerzel_windows = false; if cfg!(target_os = "macos") { let _ = app.update(Msg::KontextZu); let _ = app.update(Msg::Taste(Key::Character("d".into()), Modifiers::ALT)); assert!(app.kontext.is_none(), "macOS: Alt+D öffnet kein Menü"); assert!(app.alt_menue_buchstaben().is_empty(), "macOS: nichts reserviert") }
        // Auslösen mit der Maus: der Befehl läuft, das Menü schließt; „Ziel“ eines Dateibefehls ist nie der Menüname
        let _ = app.update(Msg::MenueAuf("Ansicht".into())); let vorher = app.einst.schema; let _ = app.update(Msg::KontextWahl(Cmd::HellDunkel)); assert!(app.kontext.is_none() && app.ziel.is_none() && app.einst.schema != vorher, "Befehl ausgelöst, Menü zu");
        // ausgegraute Einträge nennen den Grund und lösen nichts aus (kein Projekt/kein Quelltext)
        app.tabs.clear(); app.editoren.clear(); let _ = app.update(Msg::MenueAuf("Bearbeiten".into())); let (z, _) = app.kontext_zeilen(app.kontext.as_ref().unwrap());
        let ausg: Vec<&Zeile<Msg>> = z.iter().filter(|x| !x.aktiv && x.msg.is_some()).collect(); assert!(ausg.iter().all(|x| x.name.contains('(') && x.name.ends_with(')')), "Grund steht dabei: {:?}", ausg.iter().map(|x| &x.name).collect::<Vec<_>>());
        // Zen blendet die Leiste aus, Einstellung „aus“ auch
        let _ = app.update(Msg::KontextZu); let _ = app.befehl(Cmd::Zen); assert!(!app.menue_sichtbar(), "Zen: ausgeblendet"); let _ = app.befehl(Cmd::Zen); assert!(app.menue_sichtbar());
        app.menue_test = false; let _ = app.update(Msg::SetMenueleiste(false)); assert!(!app.menue_sichtbar()); let _ = app.update(Msg::SetMenueleiste(true)); assert_eq!(app.menue_sichtbar(), !cfg!(target_os = "macos"));
        // schmales Fenster: ein Knopf „Menü“, der alle Menüs untereinander zeigt (kein Überlauf)
        app.menue_test = true; let t: Vec<String> = knoten_sammeln(&mut app, 500, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect(); assert!(t.iter().any(|x| x.contains("Menü")) && !t.iter().any(|x| x == "Bearbeiten"), "schmal: nur ‚Menü‘");
        let _ = app.update(Msg::MenueAuf("*".into())); let (z, _) = app.kontext_zeilen(app.kontext.as_ref().unwrap()); for m in befehle::MENUES { assert!(z.iter().any(|x| x.name == m.to_uppercase() && x.msg.is_none()), "Überschrift {m}") }
        assert_eq!(z.iter().filter(|x| x.msg.is_some()).count(), befehle::BEFEHLE.iter().filter(|i| befehle::MENUES.contains(&i.menue)).count());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Kleine Lücken der Doku-Runde: Taste C im Sprite-Studio, „Alles auswählen“, Alles falten/entfalten, Menüs auf Probleme und Protokoll, „Vervollständigung öffnen“
    #[test] fn kleine_luecken_taste_c_alles_auswaehlen_falten_probleme_protokoll_vervollstaendigung() {
        use iced::keyboard::{Key, Modifiers};
        // Taste C schaltet das Frame zwischen Hires und Multicolor um (Sprites mit beiden Modi); mit Cmd/Strg/Alt nicht
        let tmp = kopie_von_beispiel("examples/c64/jumprun"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false;
        app.oeffne("graphics/hero_base.vcsprite"); let rel = app.name().unwrap(); let mc = |a: &App| a.studios[&rel].doc.ist_multicolor(a.studios[&rel].doc.fi); assert!(app.studios[&rel].doc.frame_modes.len() > 1);
        let vorher = mc(&app); let _ = app.update(Msg::Taste(Key::Character("c".into()), Modifiers::empty())); assert_eq!(mc(&app), !vorher, "C schaltet um"); let _ = app.update(Msg::Taste(Key::Character("c".into()), Modifiers::empty())); assert_eq!(mc(&app), vorher, "C schaltet zurück");
        assert!(app.studio_aktionen().iter().any(|a| a.name == "Modus: Multicolor" && a.kuerzel == "C"), "Befehlspalette nennt C");
        // Editor: Alles auswählen (Kontextmenü), Alles falten/entfalten, Vervollständigung öffnen
        std::fs::write(tmp.join("t.asm"), ".macro foo\n  nop\n  nop\n.endmacro\nstart: ld\n").unwrap(); app.oeffne("t.asm");
        let _ = app.update(Msg::KontextAuf(Art::Editor, iced::Point::new(500.0, 200.0), None)); let (z, _) = app.kontext_zeilen(app.kontext.as_ref().unwrap());
        for n in ["Alles auswählen", "Alles falten", "Alles entfalten"] { assert!(z.iter().any(|x| x.name == n && x.aktiv), "{n} im Kontextmenü des Editors"); }
        let _ = app.update(Msg::KontextZu); let _ = app.befehl(Cmd::AlleAuswaehlen); assert!(app.ed().unwrap().selection().is_some(), "alles ausgewählt");
        let anzahl = { let e = app.ed().unwrap(); (0..e.lines.len()).filter(|l| e.fold_range(*l).is_some()).count() }; assert!(anzahl >= 1, "Testtext hat faltbare Stellen");
        let _ = app.befehl(Cmd::AllesFalten); assert_eq!(app.ed().unwrap().folds.len(), anzahl, "alles gefaltet"); let _ = app.befehl(Cmd::AllesEntfalten); assert!(app.ed().unwrap().folds.is_empty(), "alles entfaltet");
        { let e = app.ed().unwrap(); e.anchor = None; e.cursor = Pos { line: 4, col: 8 }; } let _ = app.befehl(Cmd::Vervollstaendigen); assert!(app.ed().unwrap().completion.as_ref().is_some_and(|c| c.items.iter().any(|i| i == "lda")), "Vorschläge für „ld“");
        // Probleme: Rechtsklick auf eine Meldung, Protokoll: Rechtsklick
        app.probleme = vec![Problem { datei: "t.asm".into(), zeile: 5, text: "E202: unbekanntes Symbol".into(), fehler: true }, Problem { datei: "t.asm".into(), zeile: 1, text: "W1: Warnung".into(), fehler: false }];
        let _ = app.update(Msg::KontextAuf(Art::Problem, iced::Point::new(300.0, 700.0), Some("1".into()))); let (z, _) = app.kontext_zeilen(app.kontext.as_ref().unwrap());
        for n in ["Problem: Zur Zeile springen", "Problem: Meldung kopieren", "Alle Probleme kopieren"] { assert!(z.iter().any(|x| x.name == n && x.aktiv), "{n}") }
        let _ = app.update(Msg::KontextWahl(Cmd::ProblemSpringen)); assert_eq!(app.ed().unwrap().cursor.line, 0, "springt zu Zeile 1 des gewählten Problems");
        app.log = vec!["a".into(), "b".into()]; let _ = app.update(Msg::KontextAuf(Art::Protokoll, iced::Point::new(300.0, 700.0), None)); let (z, _) = app.kontext_zeilen(app.kontext.as_ref().unwrap()); assert!(z.iter().any(|x| x.name.starts_with("Protokoll leeren")) && z.iter().any(|x| x.name == "Protokoll kopieren"));
        let _ = app.update(Msg::KontextWahl(Cmd::ProtokollLeeren)); assert!(app.log.is_empty(), "Protokoll geleert");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Prettyprint beim Verlassen der Zeile, Befehle Zeile/Datei formatieren, Palettenbefehle Schrift und Darstellung
    #[test] fn prettyprint_beim_verlassen_der_zeile_und_palettenbefehle() {
        use vc_gui_editor::core::Pos;
        let tmp = kopie_von_beispiel("examples/c64/prallball"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone()));
        std::fs::write(tmp.join("pp.asm"), "start:\n  nop\n  rts\n").unwrap(); app.oeffne("pp.asm");
        // Zeile 1 verunstalten, dann in Zeile 2 gehen: Zeile 1 wird beim Verlassen formatiert, der Cursor bleibt in Zeile 2
        { let ed = app.ed().unwrap(); ed.cursor = Pos { line: 1, col: 5 }; ed.anchor = None; }
        let _ = app.update(Msg::Ed(EdMsg::Insert(" ".into()))); let _ = app.update(Msg::Ed(EdMsg::Insert(" ".into()))); let _ = app.update(Msg::Ed(EdMsg::Insert("x".into())));
        assert!(app.ed().unwrap().lines[1].contains("  "), "noch in der Zeile: nichts angefasst: {:?}", app.ed().unwrap().lines[1]);
        let _ = app.update(Msg::Ed(EdMsg::Move { dx: 0, dy: 1, extend: false }));
        let z1 = app.ed().unwrap().lines[1].clone(); assert_eq!(z1, "        nop x", "formatiert beim Verlassen: {z1:?}"); assert_eq!(app.ed().unwrap().cursor.line, 2);
        // Einstellung aus: nichts passiert
        let _ = app.update(Msg::SetPretty(false)); { let ed = app.ed().unwrap(); ed.cursor = Pos { line: 2, col: 0 }; }
        let _ = app.update(Msg::Ed(EdMsg::Insert(" ".into()))); let _ = app.update(Msg::Ed(EdMsg::Insert(" ".into()))); let _ = app.update(Msg::Ed(EdMsg::Move { dx: 0, dy: -1, extend: false }));
        assert!(app.ed().unwrap().lines[2].starts_with("  "), "aus: unverändert"); let _ = app.update(Msg::SetPretty(true));
        // Mit Auswahl wird nicht formatiert
        { let ed = app.ed().unwrap(); ed.cursor = Pos { line: 2, col: 3 }; ed.anchor = None; }
        let _ = app.update(Msg::Ed(EdMsg::Insert("    ".into())));
        { let ed = app.ed().unwrap(); ed.anchor = Some(Pos { line: 2, col: 0 }); ed.cursor = Pos { line: 0, col: 0 }; }
        let _ = app.update(Msg::Ed(EdMsg::Move { dx: 0, dy: 0, extend: true }));
        // Befehle: Zeile und Datei formatieren, Datei als ein Undo-Schritt
        { let ed = app.ed().unwrap(); ed.anchor = None; ed.lines[2] = "   rts      ; ende".into(); ed.lines[1] = "    nop".into(); ed.cursor = Pos { line: 0, col: 0 }; }
        let _ = app.befehl(Cmd::DateiFormatieren); let t = app.ed().unwrap().text(); assert_eq!(t, "start:\n        nop\n        rts                             ; ende\n");
        app.ed().unwrap().undo(); assert!(app.ed().unwrap().text().contains("    nop") || app.ed().unwrap().text().contains("   rts"), "ein Undo stellt die Datei her");
        let _ = app.befehl(Cmd::DateiFormatieren); let _ = app.befehl(Cmd::DateiFormatieren); assert!(app.hinweis.contains("schon formatiert"), "{:?}", app.hinweis);
        // Palettenbefehle: Schrift größer/kleiner, Darstellung zurücksetzen
        let s0 = app.einst.editor_schrift; let _ = app.befehl(Cmd::EditorSchriftGroesser); assert_eq!(app.einst.editor_schrift, s0 + 1.0); let _ = app.befehl(Cmd::EditorSchriftKleiner); let _ = app.befehl(Cmd::EditorSchriftKleiner); assert_eq!(app.einst.editor_schrift, s0 - 1.0);
        let _ = app.update(Msg::SetZoom(1.5)); let _ = app.befehl(Cmd::Zen); let _ = app.befehl(Cmd::DarstellungZuruecksetzen);
        assert_eq!((app.einst.zoom, app.einst.editor_schrift, app.layout.zen()), (1.0, 13.0, false), "Darstellung zurückgesetzt");
        for c in [Cmd::EditorSchriftGroesser, Cmd::EditorSchriftKleiner, Cmd::DarstellungZuruecksetzen, Cmd::ZeileFormatieren, Cmd::DateiFormatieren] { assert!(vc_gui_ui::befehle::info(c).tasten.is_empty(), "keine Kürzel: nur Palette und Menü"); }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Handtest-Fund: Cmd+Umschalt+B (Zen) im Tracker-Studio und Tastaturlayout-Beschriftung per Klick in den Einstellungen
    #[test] fn kuerzel_mit_umschalt_gehen_im_tracker_durch_und_layoutwahl_wirkt_auf_die_beschriftung() {
        use iced::{mouse, Event, Point};
        let (mut app, tmp) = song_app(); let p = Point::new(600.0, 300.0);
        let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]);
        for tasten in ["Cmd+Umschalt+B", "Cmd+Umschalt+O", "Cmd+Umschalt+F", "Cmd+Umschalt+E", "Cmd+Umschalt+Z", "Cmd+B", "Cmd+Alt+B"] {
            let (ev, k, mods) = kuerzel_ev(tasten); assert!(trifft(tasten, &k, mods), "{tasten} erkannt");
            let (msgs, st) = ui_status(&mut app, ev, p);
            assert!(st.iter().all(|s| *s == iced::event::Status::Ignored), "Tracker darf {tasten} nicht verbrauchen: {st:?}");
            let _ = msgs;
        }
        // die Nachricht, die die Oberfläche daraus macht, schaltet den Zen-Modus
        let (_, k, mods) = kuerzel_ev("Cmd+Umschalt+B"); assert!(!app.layout.zen()); let _ = app.update(Msg::TasteP(k.clone(), mods, iced::keyboard::key::Physical::Unidentified(iced::keyboard::key::NativeCode::Unidentified))); assert!(app.layout.zen(), "Cmd+Umschalt+B im Tracker schaltet Zen ein");
        let _ = app.update(Msg::TasteP(k, mods, iced::keyboard::key::Physical::Unidentified(iced::keyboard::key::NativeCode::Unidentified))); assert!(!app.layout.zen(), "und wieder aus");
        // großes „B“ (macOS meldet mit Umschalt manchmal den Großbuchstaben)
        let mods = iced::keyboard::Modifiers::COMMAND | iced::keyboard::Modifiers::SHIFT; let _ = app.update(Msg::Taste(iced::keyboard::Key::Character("B".into()), mods)); assert!(app.layout.zen(), "großes B"); let _ = app.update(Msg::Taste(iced::keyboard::Key::Character("B".into()), mods));
        // Tastaturlayout: Klick auf „QWERTY“ in Einstellungen ▸ Editor ändert die Einstellung und die Beschriftung der Klaviaturkarte
        let _ = app.befehl(Cmd::Einstellungen); let _ = app.update(Msg::Set(setui::SetAkt::Seite(setui::Seite::Editor)));
        for (name, soll) in [("QWERTY", einstellungen::Tastaturlayout::Qwerty), ("AZERTY", einstellungen::Tastaturlayout::Azerty), ("QWERTZ", einstellungen::Tastaturlayout::Qwertz)] {
            let (x, y) = ort(&mut app, name); let q = Point::new(x, y);
            let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: q }), q), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), q), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), q)]);
            assert_eq!(app.einst.tastaturlayout, soll, "Klick auf {name} übernimmt die Einstellung"); assert_eq!(app.layout(), soll);
        }
        let _ = app.update(Msg::SetLayout(einstellungen::Tastaturlayout::Qwerty)); app.oeffne("music/tune.vcsong"); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect();
        assert!(t.iter().any(|x| x.contains("Computertasten Z S X")), "Beschriftung folgt QWERTY: {:?}", t.iter().filter(|x| x.contains("Computertasten")).collect::<Vec<_>>());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// K2: Export-Dialog von Bank und Kachelsatz hat den Block „Kachel-Layout“ mit Speicherbedarf und tiles_use-Zeile; die Wahl wirkt auf die Dateien
    #[test] fn export_dialog_kachel_layout_wirkt_auf_vorschau_speicher_tiles_use_und_dateien() {
        let (mut app, tmp) = kachel_app(); app.oeffne("graphics/level.vcscreen"); assert!(app.banken.contains_key("graphics/level.vcscreen")); let _ = app.befehl(Cmd::Exportieren);
        let d = app.export.as_ref().expect("Dialog"); assert_eq!(d.kachel, Some(vc_media::tiles::TilesLayout::default())); assert!(d.cache.tiles_use.starts_with("tiles_use level_tiles_c0,"), "{}", d.cache.tiles_use);
        let (a, z) = d.speicher.clone().expect("Speicher"); assert_eq!(z.karten, a.karten * 2);
        let k = knoten_sammeln(&mut app, 1280, 900); let t: Vec<&str> = k.iter().filter(|k| k.art == "text").map(|k| k.text.as_str()).collect();
        assert!(t.contains(&"Kachel-Layout") && t.contains(&"interleaved") && t.contains(&"spaltenweise") && t.contains(&"16 Bit"), "{t:?}"); assert!(t.iter().any(|x| x.starts_with("Speicher ") && x.contains("mit 16-Bit-Index")), "Speicherzeile");
        let _ = app.update(Msg::ExportFormat("bin".into())); let _ = app.update(Msg::ExportKachel(exportdlg::KachelAkt::Index(true))); let _ = app.update(Msg::ExportKachel(exportdlg::KachelAkt::Karte(true)));
        assert!(app.export.as_ref().unwrap().cache.files.iter().any(|f| f.0.contains("map0_hi")), "Hochbyte-Ebene in der Vorschau: {:?}", app.export.as_ref().unwrap().cache.files);
        let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Speichern)); assert!(tmp.join("build/export/level.map0_hi.bin").is_file(), "Datei mit 16-Bit-Index geschrieben: {:?}", app.log.last());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn aktive_datei_bauen_starten_debuggen_haben_eigene_befehle_und_kuerzel() {
        for (c, k) in [(Cmd::BauenAktiv, "Alt+Umschalt+F7"), (Cmd::StartenAktiv, "Alt+Umschalt+F5"), (Cmd::DebuggenAktiv, "Alt+Umschalt+F6")] { assert_eq!(befehle::info(c).tasten, k); let (_, key, mods) = kuerzel_ev(k); assert!(trifft(k, &key, mods)); assert_eq!(befehle::BEFEHLE.iter().filter(|i| i.tasten == k).count(), 1, "{k} nur einmal vergeben"); }
        let (mut app, _) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("sprite_demo.asm");
        let _ = app.befehl(Cmd::StartenAktiv); assert_eq!(app.danach, Danach::Start); assert!(app.baut, "baut die aktive Datei"); app.baut = false;
        let _ = app.befehl(Cmd::BauenAktiv); assert_eq!(app.danach, Danach::Nichts); app.baut = false;
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn assistent_teillaeufe_und_nach_neuem_suchen_sind_befehle_im_hilfe_menue() {
        let (mut app, _) = studio_app(); einstellungen::test_konfig(std::env::temp_dir().join(format!("vcgui-wizcmd-{}", std::process::id())));
        for c in [Cmd::AssistentRoms, Cmd::AssistentEmulatoren, Cmd::AssistentHardware, Cmd::AssistentDesign, Cmd::AssistentNeues] { assert_eq!(befehle::info(c).menue, "Hilfe"); app.wiz = None; let _ = app.befehl(c); assert!(app.wiz.is_some(), "{c:?} öffnet den Assistenten"); }
        assert!(befehle::filtere("Neuem").iter().any(|i| i.cmd == Cmd::AssistentNeues), "in der Palette auffindbar");
    }
    #[test] fn markdown_schreibhilfen_wirken_auf_auswahl_und_zeile_als_ein_rueckgaengig_schritt() {
        let (mut app, _) = studio_app(); let tmp = std::env::temp_dir().join(format!("vcgui-mdh-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap(); std::fs::write(tmp.join("n.md"), "hallo welt\nzweite\n").unwrap(); std::fs::write(tmp.join("a.asm"), "nop\n").unwrap();
        app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("n.md");
        let w = |app: &mut App| app.editoren["n.md"].text();
        // Wort am Cursor fett, noch einmal: wieder normal
        app.editoren.get_mut("n.md").unwrap().set_cursor(Pos { line: 0, col: 2 }, false); let _ = app.befehl(Cmd::MdFett); assert_eq!(w(&mut app), "**hallo** welt\nzweite\n"); let _ = app.befehl(Cmd::MdFett); assert_eq!(w(&mut app), "hallo welt\nzweite\n");
        // Auswahl kursiv, dann Rückgängig = ein Schritt
        { let e = app.editoren.get_mut("n.md").unwrap(); e.set_cursor(Pos { line: 0, col: 6 }, false); e.set_cursor(Pos { line: 0, col: 10 }, true); } let _ = app.befehl(Cmd::MdKursiv); assert_eq!(w(&mut app), "hallo *welt*\nzweite\n");
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(w(&mut app), "hallo welt\nzweite\n", "ein Schritt");
        // Listen, Überschrift, Zitat, Tabelle
        app.editoren.get_mut("n.md").unwrap().set_cursor(Pos { line: 1, col: 0 }, false); let _ = app.befehl(Cmd::MdAufzaehlung); assert!(w(&mut app).contains("- zweite")); let _ = app.befehl(Cmd::MdAufgaben); assert!(w(&mut app).contains("- [ ] zweite"), "{}", w(&mut app));
        let _ = app.befehl(Cmd::MdAufgabeUmschalten); assert!(w(&mut app).contains("- [x] zweite"));
        app.editoren.get_mut("n.md").unwrap().set_cursor(Pos { line: 0, col: 0 }, false); let _ = app.befehl(Cmd::MdUeberschriftTiefer); assert!(w(&mut app).starts_with("# hallo"), "{}", w(&mut app)); let _ = app.befehl(Cmd::MdUeberschriftHoeher); assert!(w(&mut app).starts_with("hallo"));
        let _ = app.befehl(Cmd::MdTabelleNeu); assert!(w(&mut app).contains("| Spalte 1"), "{}", w(&mut app));
        // nur bei Markdown nutzbar; .asm: nichts passiert; „als Markdown behandeln“ schaltet um
        app.oeffne("a.asm"); assert!(!app.nutzbar(Cmd::MdFett)); let _ = app.befehl(Cmd::MdFett); assert_eq!(app.editoren["a.asm"].text(), "nop\n"); assert!(app.nutzbar(Cmd::MdAlsMarkdown)); let _ = app.befehl(Cmd::MdAlsMarkdown); assert!(app.nutzbar(Cmd::MdFett), "als Markdown behandelt"); let _ = app.befehl(Cmd::MdAlsMarkdown); assert!(!app.nutzbar(Cmd::MdFett));
        assert_eq!(befehle::info(Cmd::MdKursiv).tasten, "Cmd+I"); let _ = std::fs::remove_dir_all(&tmp);
    }
    /// V2: Dialog „Zeichensatz hinzufügen“ – eigener Satz als Kopie, ROM-Import (eigene Datei) und PNG-Raster, nur im Temp-Projekt
    #[test] fn zeichensatz_hinzufuegen_dialog_standard_rom_und_png_legen_dateien_an() {
        use zeichendlg::{ZAkt, ZTab};
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/gfx_demo"); einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden();
        let akt = |app: &mut App, a: ZAkt| { let _ = app.update(Msg::Zeichen(a)); };
        let _ = app.befehl(Cmd::StdZeichensatz); assert!(app.zeichen.is_some()); let k = knoten_sammeln(&mut app, 1280, 900); assert!(k.iter().any(|k| k.text == "Zeichensatz hinzufügen") && k.iter().any(|k| k.text == "Aus PNG-Raster"), "Dialog mit drei Reitern");
        let i = app.zeichen.as_ref().unwrap().liste.iter().position(|e| e.eigen.as_deref() == Some("vc-c64-upper")).expect("eigener C64-Satz in der Liste"); akt(&mut app, ZAkt::Std(i)); akt(&mut app, ZAkt::Ok);
        assert!(app.zeichen.is_none(), "{:?}", app.log.last()); assert!(tmp.join("graphics/vc-standard/vc-c64-upper.vcchar").is_file() && tmp.join("graphics/vc-standard/HERKUNFT.txt").is_file(), "Kopie mit Herkunft");
        // PNG-Raster: aus font.vcchar ein Raster schreiben und als neuen Satz einlesen
        let png = tmp.join("raster.png"); vc_media::project::export_sheet(&tmp, &app.data, "graphics/font.vcchar", &png).unwrap();
        let _ = app.befehl(Cmd::ZeichensatzAusPng); assert_eq!(app.zeichen.as_ref().unwrap().tab, ZTab::Png); assert!(app.zeichen.as_ref().unwrap().problem().is_some(), "ohne Datei gesperrt");
        akt(&mut app, ZAkt::PngGewaehlt(Some(png.clone()))); akt(&mut app, ZAkt::Name("neu_png".into())); assert!(app.zeichen.as_ref().unwrap().problem().is_none(), "{:?}", app.zeichen.as_ref().unwrap().problem()); akt(&mut app, ZAkt::Ok);
        assert!(tmp.join("graphics/neu_png.vcchar").is_file(), "{:?}", app.log.last());
        // aus ROM: eine selbst gemachte 4-KB-Datei (nur Größe, nicht erkannt) wird nach Größe vorgeschlagen
        let rom = tmp.join("eigen.rom"); std::fs::write(&rom, vec![0x18u8; 4096]).unwrap();
        let _ = app.befehl(Cmd::ZeichensatzAusRom); akt(&mut app, ZAkt::RomGewaehlt(Some(rom))); assert!(app.zeichen.as_ref().unwrap().erkennung.as_ref().is_some_and(|e| !e.sets.is_empty()), "Vorschläge nach Größe");
        akt(&mut app, ZAkt::Name("aus_rom".into())); akt(&mut app, ZAkt::Ok); assert!(tmp.join("graphics/aus_rom.vcchar").is_file(), "{:?} {:?}", app.log.last(), app.zeichen.as_ref().map(|z| z.meldung.clone()));
        akt(&mut app, ZAkt::Abbruch); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn hex_vorschlaege_nur_gewaehlte_zeilen_ein_rueckgaengig_schritt() {
        let (mut app, _) = studio_app(); let tmp = std::env::temp_dir().join(format!("vcgui-hexv-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("p.bas"), "10 POKE 53280,0\n20 POKE 1024,1\n30 X=PEEK(56320)\n").unwrap(); std::fs::write(tmp.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"targets":[{"id":"c64","system":"c64"}],"sources":[{"targets":["c64"],"kind":"basic","files":["p.bas"]}],"main":"p.bas"}"#).unwrap();
        einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("p.bas");
        let _ = app.befehl(Cmd::HexVorschlaege); let d = app.hex.as_ref().expect("Dialog"); assert_eq!(d.vorschlaege.len(), 2, "{:?}", d.vorschlaege.iter().map(|v| &v.to).collect::<Vec<_>>());
        let _ = app.update(Msg::HexVorschlag(hexdlg::HAkt::Ok)); assert!(app.editoren["p.bas"].text().contains("53280"), "ohne Haken ändert sich nichts");
        let _ = app.befehl(Cmd::HexVorschlaege); let _ = app.update(Msg::HexVorschlag(hexdlg::HAkt::Zeile(0, true))); let _ = app.update(Msg::HexVorschlag(hexdlg::HAkt::Ok));
        let t = app.editoren["p.bas"].text(); assert!(t.contains("$D020") && t.contains("56320"), "nur die gewählte Zeile: {t}"); let _ = app.befehl(Cmd::Rueckgaengig); assert!(app.editoren["p.bas"].text().contains("53280"), "ein Schritt");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn art_des_reiters_laesst_sich_umschalten_ohne_die_datei_zu_aendern() {
        let (mut app, _) = studio_app(); let tmp = std::env::temp_dir().join(format!("vcgui-art-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap(); std::fs::write(tmp.join("x.txt"), "nop\n").unwrap();
        app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("x.txt"); use vc_ide_model::lang::Lang; assert_eq!(app.editoren["x.txt"].lang, Lang::Plain);
        let _ = app.befehl(Cmd::ArtAsm); assert_eq!(app.editoren["x.txt"].lang, Lang::Asm); let _ = app.befehl(Cmd::ArtBasic); assert_eq!(app.editoren["x.txt"].lang, Lang::Basic); let _ = app.befehl(Cmd::ArtText); assert_eq!(app.editoren["x.txt"].lang, Lang::Plain);
        assert_eq!(std::fs::read_to_string(tmp.join("x.txt")).unwrap(), "nop\n"); let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Handtest-Wunsch: Hilfe-Themen als gerendertes Markdown im Panel und im Reiter, Links zwischen Themen, Zurück
    #[test] fn hilfe_themen_sind_gerendertes_markdown_im_panel_und_im_reiter_mit_links() {
        let (mut app, _) = studio_app(); let _ = app.update(Msg::HilfeOeffne("layout".into())); assert!(app.hilfe.doc.as_ref().is_some_and(|d| d.id == "layout"));
        let _ = app.befehl(Cmd::Hilfe); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 900).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect();
        assert!(t.iter().any(|x| x == "Im Reiter öffnen") && !t.iter().any(|x| x.starts_with("## ")), "Knopf da, keine Markdown-Zeichen sichtbar: {:?}", t.iter().take(30).collect::<Vec<_>>());
        let _ = app.befehl(Cmd::HilfeImReiter); assert!(matches!(app.tabs.get(app.aktiv), Some(Tab::Hilfe(k)) if k == "layout"));
        let t: Vec<String> = knoten_sammeln(&mut app, 1280, 900).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect(); assert!(t.iter().any(|x| x.contains("Oberfläche anpassen")), "Reitertitel: {:?}", t.iter().take(14).collect::<Vec<_>>());
        let _ = app.update(Msg::MdLink("help:settings".into())); assert!(matches!(app.tabs.get(app.aktiv), Some(Tab::Hilfe(k)) if k == "settings"), "Link wirkt im selben Reiter");
        let _ = app.update(Msg::MdLink("https://example.org".into())); assert!(matches!(app.tabs.get(app.aktiv), Some(Tab::Hilfe(k)) if k == "settings"), "Weblinks öffnen nichts");
    }
    #[test] #[ignore] fn bild_hilfe() { let (mut app, _) = studio_app(); let _ = app.update(Msg::HilfeOeffne("layout".into())); let _ = app.befehl(Cmd::Hilfe); foto(&mut app, &[], "/tmp/vc-hilfe-panel.png"); let _ = app.befehl(Cmd::HilfeImReiter); foto(&mut app, &[], "/tmp/vc-hilfe-reiter.png"); }
    #[test] fn dasselbe_kuerzel_als_menue_und_tastenereignis_wirkt_nur_einmal() {
        let (mut app, _) = App::neu(); app.doppel_ms = 10_000; app.layout = Layout::vorgabe();
        let (_, k, mods) = kuerzel_ev("Cmd+Umschalt+B");
        let _ = app.update(Msg::Menue("zen".into())); assert!(app.layout.zen()); let _ = app.update(Msg::Taste(k.clone(), mods)); assert!(app.layout.zen(), "zweites Ereignis desselben Kürzels (Taste nach Menü) wird verworfen");
        let _ = app.update(Msg::Taste(k.clone(), mods)); assert!(!app.layout.zen(), "gleicher Weg zweimal hintereinander gilt als bewusstes zweites Drücken");
    }
    #[test] fn tracker_studio_klicken_tippen_auswahl_block_speichern_mit_echten_ereignissen() {
        use iced::{keyboard::{key::Named, Key, Modifiers}, mouse, Event, Point};
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong";
        assert!(app.songs.contains_key(rel) && app.name().as_deref() == Some(rel) && app.editoren.len() == 1, "Song öffnet im Tracker-Studio"); let _ = simuliere(&mut app, &[]);
        let klick = |app: &mut App, x: f32, y: f32| { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]); };
        app.songs.get_mut(rel).unwrap().leisten_zu = Some(true); let _ = simuliere(&mut app, &[]); let (ox, oy) = scroll_ecke(&mut app, 300.0); let kb = app.songs[rel].kb.get(); let zelle = move |c: usize, r: usize| (ox + 34.0 + c as f32 * kb + kb / 2.0, oy + r as f32 * 20.0 + 10.0);   // die Kanalköpfe stehen über dem Raster, nicht darin
        let taste = |app: &mut App, z: &str| { let (ev, key, mods) = kuerzel_ev(z); let (msgs, st) = ui_status(app, ev, Point::new(500.0, 400.0)); for m in msgs { let _ = app.update(m); } if st.iter().all(|s| *s == iced::event::Status::Ignored) { let _ = app.update(Msg::Taste(key, mods)); } };
        // Klick setzt den Cursor
        let (x, y) = zelle(1, 3); klick(&mut app, x, y); assert_eq!(app.songs[rel].doc.cursor, (3, 1), "Klick auf Zeile 3, Kanal 2");
        // Noten tippen (Klaviertasten): Z = C, X = D; Cursor rückt vor; Oktave mit . ; Instrument mit ]
        taste(&mut app, "Z"); taste(&mut app, "X"); assert_eq!((app.songs[rel].doc.zelle(0, 3, 1).as_str(), app.songs[rel].doc.zelle(0, 4, 1).as_str(), app.songs[rel].doc.cursor), ("C-3 00", "D-3 00", (5, 1)));
        let _ = app.update(Msg::Taste(Key::Character(".".into()), Modifiers::empty())); let _ = app.update(Msg::Taste(Key::Character("]".into()), Modifiers::empty())); taste(&mut app, "Z"); assert_eq!(app.songs[rel].doc.zelle(0, 5, 1), "C-4 01", "Oktave 4, Instrument 1");
        taste(&mut app, "1"); assert_eq!(app.songs[rel].doc.zelle(0, 6, 1), "=== --", "Taste 1 = Notenende");
        let _ = app.update(Msg::Taste(Key::Named(Named::ArrowUp), Modifiers::empty())); let _ = app.update(Msg::Taste(Key::Named(Named::ArrowUp), Modifiers::empty())); let _ = app.update(Msg::Taste(Key::Named(Named::Delete), Modifiers::empty())); assert_eq!(app.songs[rel].doc.zelle(0, 5, 1), "--- --", "Entf löscht");
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.songs[rel].doc.zelle(0, 5, 1), "C-4 01", "Cmd+Z");
        // Auswahl mit Umschalt+Pfeil, Cmd+C, Cursor weg, Cmd+V
        let (x, y) = zelle(1, 3); klick(&mut app, x, y); let _ = app.update(Msg::Taste(Key::Named(Named::ArrowDown), Modifiers::SHIFT)); let _ = app.update(Msg::Taste(Key::Named(Named::ArrowDown), Modifiers::SHIFT)); assert_eq!(app.songs[rel].doc.auswahl, Some(((3, 1), (5, 1))));
        taste(&mut app, "Cmd+C"); assert!(app.songs[rel].doc.status.starts_with("Kopiert (3×1")); let (x, y) = zelle(2, 10); klick(&mut app, x, y); taste(&mut app, "Cmd+V"); assert_eq!((app.songs[rel].doc.zelle(0, 10, 2).as_str(), app.songs[rel].doc.zelle(0, 12, 2).as_str()), ("C-3 00", "C-4 01"), "Block eingefügt");
        // Ziehen wählt aus; Speichern per Befehl; Export-Knopf
        let (p0, p1) = (Point::new(zelle(0, 9).0, zelle(0, 9).1), Point::new(zelle(2, 11).0, zelle(2, 11).1)); let msgs = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p0 }), p0), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p0), (Event::Mouse(mouse::Event::CursorMoved { position: p1 }), p1), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p1)]);; let _ = msgs;
        assert_eq!(app.songs[rel].doc.auswahl, Some(((9, 0), (11, 2))), "Ziehen mit der Maus wählt aus");
        let _ = app.befehl(Cmd::Speichern); assert!(!app.songs[rel].doc.dirty()); assert!(std::fs::read_to_string(tmp.join(rel)).unwrap().contains("C-4 01"));
        let _ = simuliere(&mut app, &[]); let _ = app.befehl(Cmd::Exportieren); assert_eq!(app.export.as_ref().map(|d| d.rel.clone()).as_deref(), Some(rel), "{:?}", app.log.last()); assert!(app.export.as_ref().unwrap().cache.error.is_none(), "{:?}", app.export.as_ref().unwrap().cache.error);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_studio_muster_reihenfolge_instrumente_und_kanaele_mit_klicks() {
        use iced::{mouse, Event, Point}; use songstudio::GAkt;
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]);
        let klick = |app: &mut App, x: f32, y: f32| { let p = Point::new(x, y); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]); };
        let akt = |app: &mut App, a: GAkt| { let _ = app.update(Msg::SongAkt(rel.into(), a)); };
        // Kanäle: Stumm/Solo per Klick
        akt(&mut app, GAkt::Stumm(0)); assert!(app.songs[rel].doc.stumm[0], "Stumm 1"); akt(&mut app, GAkt::Solo(0)); assert!(app.songs[rel].doc.solo[0], "Solo 1"); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect(); assert!(t.iter().any(|x| x == "Stumm ✓") && t.iter().any(|x| x == "Solo ✓"), "Zustand im Knopf: {t:?}");
        // Muster neu (Knopf), Eintrag der Reihenfolge wählen und auf das Muster setzen
        assert!(knoten_sammeln(&mut app, 1280, 800).iter().any(|k| k.art == "text" && k.text == "Muster · 2"), "Musterliste mit „+“ im Kopf"); akt(&mut app, GAkt::MusterNeu); assert_eq!((app.songs[rel].doc.anzahl_muster(), app.songs[rel].doc.muster), (3, 2), "Neu legt ein Muster an"); akt(&mut app, GAkt::Modus(songstudio::SongModus::Ablauf)); let (x, y) = ort(&mut app, "02"); klick(&mut app, x, y); assert_eq!((app.songs[rel].doc.ord_wahl, app.songs[rel].doc.muster), (2, 0), "Klick auf Eintrag 02 zeigt sein Muster");
        akt(&mut app, GAkt::OrdPlus); assert_eq!(app.songs[rel].doc.reihenfolge().len(), 5); akt(&mut app, GAkt::OrdMinus); assert_eq!(app.songs[rel].doc.reihenfolge().len(), 4); akt(&mut app, GAkt::Schleife(2)); assert_eq!(app.songs[rel].doc.media["loop"], 2); akt(&mut app, GAkt::MusterLoeschen); assert_eq!(app.songs[rel].doc.anzahl_muster(), 2);
        akt(&mut app, GAkt::Modus(songstudio::SongModus::Instrumente));
        // Instrument wählen (Klick), neues anlegen, Werte per Regler-Nachrichten: ein Rückgängig-Schritt je Zug; Cmd+Z
        let (x, y) = ort(&mut app, "01 *"); klick(&mut app, x, y); assert_eq!((app.songs[rel].doc.ins_wahl, app.songs[rel].doc.instrument), (1, 1), "Klick auf Instrument 01"); akt(&mut app, GAkt::InsNeu); let n = app.songs[rel].doc.ins_wahl;
        akt(&mut app, GAkt::InsZahl("a", 3)); akt(&mut app, GAkt::InsZahl("a", 7)); akt(&mut app, GAkt::ReglerLos); akt(&mut app, GAkt::InsZahl("d", 5)); akt(&mut app, GAkt::ReglerLos); akt(&mut app, GAkt::InsName("zisch".into())); akt(&mut app, GAkt::InsWelle("noise".into()));
        let i = &app.songs[rel].doc.media["instruments"][n]; assert_eq!((i["a"].as_i64(), i["d"].as_i64(), i["name"].as_str(), i["wave"].as_str()), (Some(7), Some(5), Some("zisch"), Some("noise")));
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.songs[rel].doc.media["instruments"][n]["wave"], "pulse", "Cmd+Z nimmt die Wellenform zurück"); let _ = app.befehl(Cmd::Rueckgaengig); assert_ne!(app.songs[rel].doc.media["instruments"][n]["name"], "zisch");
        akt(&mut app, GAkt::InsLoeschen); assert_eq!(app.songs[rel].doc.anzahl_instrumente(), n, "das neue, unbenutzte Instrument lässt sich löschen"); akt(&mut app, GAkt::InsWahl(0)); akt(&mut app, GAkt::InsLoeschen); assert!(app.songs[rel].doc.status.starts_with("Löschen nur"), "benutztes Instrument bleibt");
        let _ = simuliere(&mut app, &[]); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_studio_im_muster_modi_noten_instrumente_ablauf_transport_und_kanaele_in_allen_modi_sichtbar() {
        use songstudio::SongModus;
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        assert_eq!(app.songs[rel].modus, SongModus::Noten, "Vorgabe: Noten");
        let alle_modi = ["Noten", "Instrumente", "Ablauf", "▶ Song", "▶ Muster", "Exportieren …", "Speichern", "Summe"];
        for (cmd, modus) in [(Cmd::TrackerNoten, SongModus::Noten), (Cmd::TrackerInstrumente, SongModus::Instrumente), (Cmd::TrackerAblauf, SongModus::Ablauf)] {
            let _ = app.befehl(cmd); assert_eq!(app.songs[rel].modus, modus, "Befehl wechselt den Modus"); let t = texte(&mut app);
            for n in alle_modi { assert!(t.iter().any(|x| x == n), "{n} in {modus:?} sichtbar: {t:?}") }
            assert!(t.iter().any(|x| x == "TRK"), "Kopfzeile mit Art-Chip");
            match modus {
                SongModus::Noten => { assert!(t.iter().any(|x| x.starts_with("Muster · ")) && t.iter().any(|x| x == "Werkzeug") && t.iter().any(|x| x == "gilt für neue Noten · nicht in der Datei"), "Noten: Musterliste, Karte Werkzeug „gilt für neue Noten“: {t:?}") }
                SongModus::Instrumente => { assert!(t.iter().any(|x| x.starts_with("Instrumente ·")), "Instrumente: Klang-Editor: {t:?}"); assert!(!t.iter().any(|x| x == "gilt für neue Noten · nicht in der Datei")) }
                SongModus::Ablauf => { assert!(t.iter().any(|x| x.starts_with("Reihenfolge")), "Ablauf: Reihenfolge: {t:?}"); assert!(t.iter().any(|x| x.contains("im Ablauf") || x == "unbenutzt"), "Ablauf: Musterkarten mit Nutzung: {t:?}") }
            }
        }
        // Die Modus-Wahl ändert den Fokus nicht und die Datei nicht
        assert!(!app.songs[rel].doc.dirty()); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_instrumente_modus_vorhoeren_je_instrument_klaviatur_probespielen_ohne_einzutragen_und_alt_pfeil_je_zelle() {
        use songstudio::{GAkt, SongModus}; use iced::{keyboard::{key::Named, Key, Modifiers}, mouse, Event, Point};
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]); let _ = aussen::aufgezeichnet();
        let vor = |a: &[aussen::Aufruf]| a.iter().filter(|x| matches!(x, aussen::Aufruf::AudioVorhoeren { .. })).count();
        // Alt+↑/↓ im Modus Noten: Instrument der Zelle mit Note ändern (Cursor auf C-4 00), spielt vor; leere Zelle bleibt unverändert
        { let d = &mut app.songs.get_mut(rel).unwrap().doc; d.cursor = (0, 0); } let z0 = app.songs[rel].doc.zelle(0, 0, 0); assert!(z0.starts_with("C-4"), "{z0}");
        let alt = Modifiers::ALT; let _ = aussen::aufgezeichnet(); assert!(app.song_taste(rel, &Key::Named(Named::ArrowUp), alt)); assert_eq!(app.songs[rel].doc.zelle(0, 0, 0), "C-4 01", "Alt+↑ wählt das nächste Instrument für diese Zelle"); assert_eq!(vor(&aussen::aufgezeichnet()), 1, "Vorhören der Zelle");
        assert!(app.song_taste(rel, &Key::Named(Named::ArrowDown), alt)); assert!(app.song_taste(rel, &Key::Named(Named::ArrowDown), alt)); assert_eq!(app.songs[rel].doc.zelle(0, 0, 0), "C-4 00", "nicht unter Instrument 0");
        { let d = &mut app.songs.get_mut(rel).unwrap().doc; d.cursor = (1, 0); } let leer = app.songs[rel].doc.zelle(0, 1, 0); let _ = app.song_taste(rel, &Key::Named(Named::ArrowUp), alt); assert_eq!(app.songs[rel].doc.zelle(0, 1, 0), leer, "leere Zelle bleibt leer");
        // Modus Instrumente: ▶ Vorhören je Instrument
        let _ = app.befehl(Cmd::TrackerInstrumente); let _ = aussen::aufgezeichnet(); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect();
        assert!(t.iter().any(|x| x.starts_with("Instrumente · ")) && t.iter().filter(|x| *x == "▶").count() >= 2 && t.iter().any(|x| x == "Vorhören") && !t.iter().any(|x| x == "Klaviatur"), "{t:?}");
        let (x, y) = ort(&mut app, "▶"); let p = Point::new(x, y); let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]);
        assert_eq!(vor(&aussen::aufgezeichnet()), 1, "Klick auf ▶ spielt das Instrument vor");
        // Klaviertasten spielen vor, tragen NICHTS ein; Pfeile wählen das Instrument
        let vorher = app.songs[rel].doc.media.to_string(); let _ = aussen::aufgezeichnet(); assert!(app.song_taste(rel, &Key::Character("z".into()), Modifiers::empty())); assert!(app.song_taste(rel, &Key::Character("s".into()), Modifiers::empty())); assert_eq!(vor(&aussen::aufgezeichnet()), 2); assert_eq!(app.songs[rel].doc.media.to_string(), vorher, "Probespielen ändert die Datei nicht");
        let i0 = app.songs[rel].doc.ins_wahl; assert!(app.song_taste(rel, &Key::Named(Named::ArrowDown), Modifiers::empty())); assert_eq!(app.songs[rel].doc.ins_wahl, (i0 + 1).min(app.songs[rel].doc.anzahl_instrumente() - 1), "↓ wählt das nächste Instrument");
        // Klick auf die Klaviatur spielt die Note vor
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Probe(49))); assert_eq!(vor(&aussen::aufgezeichnet()), 1); assert!(!app.songs[rel].doc.dirty(), "Vorhören macht die Datei nicht „geändert“"); let _ = app.songs[rel].modus == SongModus::Instrumente;
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_klaviatur_karte_traegt_noten_ein_rueckt_vor_und_laesst_sich_einklappen() {
        use songstudio::GAkt;
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let t = texte(&mut app); assert!(t.iter().any(|x| x == "Klaviatur") && t.iter().any(|x| x == "Kanäle") && t.iter().any(|x| x == "Pegel"), "Karten der Noten-Seitenspalte: {t:?}"); assert!(!t.iter().any(|x| x == "Muster in der Übersicht"));
        { let d = &mut app.songs.get_mut(rel).unwrap().doc; d.cursor = (1, 0); d.instrument_waehlen(1); } let _ = aussen::aufgezeichnet();
        // Klick auf eine Taste (Oktave 3: Note 49 = C-4) trägt mit dem gewählten Instrument ein und rückt um den Schritt vor
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Eintragen(49))); assert_eq!(app.songs[rel].doc.zelle(0, 1, 0), "C-4 01"); assert_eq!(app.songs[rel].doc.cursor, (2, 0), "Cursor rückt vor");
        assert!(aussen::aufgezeichnet().iter().any(|a| matches!(a, aussen::Aufruf::AudioVorhoeren { .. })), "spielt die Note vor"); assert!(app.songs[rel].doc.dirty());
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Eintragen(50))); assert_eq!(app.songs[rel].doc.zelle(0, 2, 0), "C#4 01", "Halbton über C");
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.songs[rel].doc.zelle(0, 2, 0), "--- --", "ein Rückgängig-Schritt je Taste");
        // Karte einklappen
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Karte("klaviatur"))); let t = texte(&mut app); assert!(!t.iter().any(|x| x.starts_with("Klick trägt die Note")), "eingeklappt: kein Inhalt");
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Karte("klaviatur"))); assert!(texte(&mut app).iter().any(|x| x.starts_with("Klick trägt die Note")), "wieder offen");
        // Instrumente-Modus: keine Klaviatur-Karte quer über die Fläche
        let _ = app.befehl(Cmd::TrackerInstrumente); let t = texte(&mut app); assert!(!t.iter().any(|x| x == "Klaviatur"), "Modus Instrumente ohne Klaviatur");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_klaviatur_zeigt_klingende_noten_beim_vorhoeren_tippen_und_abspielen() {
        use songstudio::GAkt; use iced::keyboard::{Key, Modifiers};
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]);
        assert!(app.songs[rel].klingt.is_empty() && !app.probe_aktiv);
        // (c) Eintippen per Computertaste: Z = C der Eingabeoktave (3) = Note 37; die Taste leuchtet, solange der Ton klingt
        let _ = app.update(Msg::Taste(Key::Character("z".into()), Modifiers::empty())); assert!(app.probe_aktiv, "Tick für die Anzeige läuft");
        app.audio.schritt(3000); let _ = app.update(Msg::SongTick); assert_eq!(app.songs[rel].klingt, vec![37], "Taste C der Oktave 3 klingt");
        // (a) Klick auf die Klaviatur: andere Note
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Eintragen(41))); app.audio.schritt(3000); let _ = app.update(Msg::SongTick); assert!(app.songs[rel].klingt.contains(&41), "Klick-Ton wird angezeigt: {:?}", app.songs[rel].klingt);
        // ausgeklungen: nichts mehr, Tick endet
        for _ in 0..3 { app.audio.schritt(44100); } let _ = app.update(Msg::SongTick); assert!(app.songs[rel].klingt.is_empty() && !app.probe_aktiv, "ausgeklungen: Anzeige leer, Tick beendet");
        // (b) Wiedergabe: die Noten der Kanäle (Zeile 0: C-4 im Kanal 1, C-2 im Kanal 2) leuchten gleichzeitig
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::SpielSong)); app.audio.schritt(2000); let _ = app.update(Msg::SongTick);
        let k = app.songs[rel].klingt.clone(); assert!(k.len() >= 2, "mehrere Kanäle gleichzeitig: {k:?}");
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Stopp)); let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Flackern beim Überfahren: Bildebenen dürfen bei Mausbewegungen kein neues Bild-Handle bekommen (neue Kennung = neue Textur = Flackern)
    #[test] fn mausbewegung_ueber_der_flaeche_erzeugt_keine_neuen_bild_handles_in_bildschirm_kachelsatz_und_bank() {
        use iced::{mouse, Event, Point};
        for (rel, art) in [("graphics/title.vcscreen", "Bildschirm"), ("graphics/level_tiles.vctiles", "Kachelsatz"), ("graphics/level.vcscreen", "Bank")] {
            let (mut app, tmp) = screen_app(); app.oeffne(rel); let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
            for i in 0..3 { let p = Point::new(430.0 + (i * 9) as f32, 260.0); let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]); } // Einschwingen (erste Ansicht in der endgültigen Größe)
            let vor = bildcache::erzeugt();
            for i in 0..40 { let p = Point::new(430.0 + (i * 9) as f32, 260.0 + (i % 7) as f32 * 20.0); let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]); if bildcache::erzeugt() != vor { eprintln!("{art}: Bewegung {i} bei {p:?} erzeugt Handle ({})", bildcache::erzeugt() - vor); } }
            assert_eq!(bildcache::erzeugt(), vor, "{art}: 40 Mausbewegungen über der Fläche erzeugen keine neuen Bild-Handles (sonst flackert sie)");
            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
    #[test] fn disk_kette_hat_schalter_legende_und_start_ende_und_laesst_sich_ausblenden() {
        use diskstudio::DAkt;
        let (mut app, tmp) = disk_app(); let mut d = vc_disk::D64::blank("KETTE", "01").unwrap(); d.add_prg("GROSS", &vec![7u8; 6000]).unwrap(); std::fs::write(tmp.join("kette.d64"), d.to_bytes()).unwrap(); app.projekt_neu_laden(); app.oeffne("kette.d64"); let rel = "kette.d64";
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::Datei(0))); assert!(app.disks[rel].kette_an && app.disks[rel].kette().len() > 2, "Kette ist da und standardmäßig sichtbar");
        let t = texte(&mut app); assert!(t.iter().any(|x| x.contains("Kette: ● Start ■ Ende → Richtung")), "Legende der Kette: {t:?}");
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::KetteAn(false))); assert!(!app.disks[rel].kette_an);
        let t = texte(&mut app); assert!(!t.iter().any(|x| x.contains("Kette: ● Start")), "ausgeblendet: keine Kettenlegende"); assert!(t.iter().any(|x| x == "Kette anzeigen"), "Schalter bleibt");
        let _ = app.update(Msg::DiskAkt(rel.into(), DAkt::KetteAn(true))); assert!(app.disks[rel].kette_an);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_klaviertasten_gelten_nach_der_tastenposition_und_hinweise_nach_dem_layout() {
        use iced::keyboard::{key::{Code, Physical}, Key, Modifiers}; use einstellungen::Tastaturlayout as L;
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let druck = |app: &mut App, logisch: &str, pos: Code| { let _ = app.update(Msg::TasteP(Key::Character(logisch.into()), Modifiers::empty(), Physical::Code(pos))); };
        { let d = &mut app.songs.get_mut(rel).unwrap().doc; d.cursor = (1, 0); d.oktave = 3; d.schritt = 1; }
        // QWERTZ: die Taste an der Position des QWERTY-Z trägt die Beschriftung „Y“ und ist die erste weiße Taste (C)
        druck(&mut app, "y", Code::KeyZ); assert_eq!(app.songs[rel].doc.zelle(0, 1, 0), "C-3 00", "QWERTZ: Taste „Y“ (Position des Z) = C");
        assert_eq!(app.layout(), L::Qwertz, "Layout wurde an den Tasten erkannt");
        // obere Reihe: Position des QWERTY-Y trägt auf QWERTZ „Z“ und ist der Ton A# der oberen Oktave (Halbton 21)
        druck(&mut app, "z", Code::KeyY); assert_eq!(app.songs[rel].doc.zelle(0, 2, 0), "A-4 00", "QWERTZ: Taste „Z“ (Position des Y) = Halbton 21");
        // QWERTY unverändert
        druck(&mut app, "z", Code::KeyZ); assert_eq!(app.songs[rel].doc.zelle(0, 3, 0), "C-3 00", "QWERTY: Z = C");
        // Hinweise: Einstellung QWERTZ → „Y S X …“ und „… T Z 7 U I“ in der Fußzeile, QWERTY → „Z S X …“
        app.einst.tastaturlayout = L::Qwertz; app.layout_gelernt = None; let t = texte(&mut app);
        use einstellungen::Tastaturlayout;
        assert!(songstudio::tasten_tipp(L::Qwertz).contains("Y S X D C V G B H N J M / Q 2 W 3 E R 5 T 6 Z 7 U I") && songstudio::tasten_tipp(L::Qwertz).contains("QWERTZ"), "QWERTZ-Tooltip");
        assert!(songstudio::tasten_tipp(L::Qwerty).contains("Z S X D C V G B H N J M / Q 2 W 3 E R 5 T 6 Y 7 U I"), "QWERTY-Tooltip");
        assert!(songstudio::tasten_tipp(L::Azerty).contains("W S X D C V G B H N J , / A 2 Z 3 E R 5 T 6 Y 7 U I"), "AZERTY-Tooltip");
        // sichtbare Texte: Karte Klaviatur (Noten) und Karte Vorhören (Instrumente) nennen die Tasten des Layouts
        app.einst.tastaturlayout = L::Qwertz; let t = texte(&mut app); assert!(t.iter().any(|x| x.contains("Computertasten Y S X") && x.contains("tun dasselbe")), "Klaviatur-Hinweis QWERTZ: {t:?}");
        let _ = app.befehl(Cmd::TrackerInstrumente); let t = texte(&mut app); assert!(t.iter().any(|x| x.starts_with("Computertasten Y S X D C V")), "Vorhören-Hinweis QWERTZ"); let _ = app.befehl(Cmd::TrackerNoten);
        app.einst.tastaturlayout = L::Qwerty; let t = texte(&mut app); assert!(t.iter().any(|x| x.contains("Computertasten Z S X") && x.contains("tun dasselbe")), "Klaviatur-Hinweis QWERTY");
        // Auto ohne Wissen: QWERTY (im Test kein Systemlayout)
        app.einst.tastaturlayout = L::Auto; assert_eq!(app.layout(), L::Qwerty);
        // Einstellungen: Layout wählbar, Beschriftung der Klaviatur abschaltbar
        let _ = app.update(Msg::SetLayout(L::Azerty)); assert_eq!(app.einst.tastaturlayout, L::Azerty); let _ = app.update(Msg::SetKlavierTasten(false)); assert!(!app.einst.tasten_auf_klaviatur); let _ = app.update(Msg::SetKlavierTasten(true));
        // Studio-Einzeltasten wie D, E, F sind logisch (Mnemonik), nicht positionsabhängig: Msg::Taste ohne Position bleibt wie bisher
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// K1-Nachtrag: Aktionen jedes Studios stehen in der Palette („Studio: …“) und im Kontextmenü (Menü-Taste; Rechtsklick bei Palette und Disk)
    #[test] fn studio_aktionen_stehen_in_palette_und_kontextmenue_fuer_alle_studios() {
        use iced::keyboard::{key::Named, Key, Modifiers};
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let pruefe = |app: &mut App, studio: &str, palettenname: &str, menuename: &str| {
            let a = app.studio_aktionen(); assert!(a.len() >= 2, "{studio}: Aktionen vorhanden"); assert!(a.iter().any(|x| x.name == palettenname), "{studio}: Aktion „{palettenname}“ in der Liste: {:?}", a.iter().map(|x| x.name.clone()).collect::<Vec<_>>());
            // Palette
            let _ = app.befehl(Cmd::Palette); let _ = app.update(Msg::PaletteFrage(palettenname.chars().take(8).collect())); let eintraege = app.palette_eintraege(); assert!(eintraege.iter().any(|e| matches!(e, Eintrag::Studio(n, _, _) if n == palettenname)), "{studio}: Palette findet „{palettenname}“"); app.palette.offen = false;
            // Kontextmenü über die Menü-Taste
            let _ = app.update(Msg::Taste(Key::Named(Named::ContextMenu), Modifiers::empty())); assert_eq!(app.kontext.as_ref().map(|k| k.art), Some(Art::FlaecheStudio), "{studio}: Menü-Taste öffnet das Studiomenü");
            let t = texte(app); assert!(t.iter().any(|x| x == menuename), "{studio}: Menüeintrag „{menuename}“: {t:?}"); let _ = app.update(Msg::Taste(Key::Named(Named::Escape), Modifiers::empty())); assert!(app.kontext.is_none());
        };
        // Bildschirm
        let (mut app, tmp) = screen_app(); app.oeffne("graphics/title.vcscreen"); let _ = simuliere(&mut app, &[]); pruefe(&mut app, "Bildschirm", "Werkzeug: Füllen", "Werkzeug: Füllen");
        // Eintrag wählen führt aus: Werkzeug Füllen
        let _ = app.update(Msg::Taste(Key::Named(Named::ContextMenu), Modifiers::empty())); let z = app.kontext_zeilen(app.kontext.as_ref().unwrap()).0; let i = z.iter().position(|x| x.name == "Werkzeug: Füllen").unwrap(); let _ = app.update(Msg::KontextAuf(Art::FlaecheStudio, iced::Point::new(500.0, 300.0), Some("graphics/title.vcscreen".into()))); let m = z[i].msg.clone().unwrap(); let _ = app.update(m); assert_eq!(app.screens["graphics/title.vcscreen"].doc.werkzeug, vc_ide_model::screendoc::Werkzeug::Fuellen, "Menüeintrag wählt das Werkzeug"); assert!(app.kontext.is_none());
        // Kachelsatz
        app.oeffne("graphics/level_tiles.vctiles"); let _ = simuliere(&mut app, &[]); pruefe(&mut app, "Kachelsatz", "Neue Kachel", "Um 90° drehen");
        // Bank
        app.oeffne("graphics/level.vcscreen"); let _ = simuliere(&mut app, &[]); pruefe(&mut app, "Bank", "Modus: Zeichen", "Neuer Screen");
        let _ = std::fs::remove_dir_all(&tmp);
        // Disk (auch Rechtsklick auf die Karte)
        let (mut app, tmp) = disk_app(); app.oeffne("spiel.d64"); let _ = simuliere(&mut app, &[]); pruefe(&mut app, "Disk", "Validieren", "Dem Folgesektor folgen");
        let _ = std::fs::remove_dir_all(&tmp);
        // Palette-Studio
        let (mut app, tmp) = palette_app(); let rel = app.name().unwrap(); let _ = simuliere(&mut app, &[]); assert!(app.paletten.contains_key(&rel)); pruefe(&mut app, "Palette", "Standardpalette laden", "Standardpalette laden");
        let _ = std::fs::remove_dir_all(&tmp);
        // Sprite und Tracker
        let (mut app, tmp) = studio_app(); let _ = simuliere(&mut app, &[]); pruefe(&mut app, "Sprite", "Horizontal spiegeln", "Horizontal spiegeln"); let _ = std::fs::remove_dir_all(&tmp);
        let (mut app, tmp) = song_app(); let _ = simuliere(&mut app, &[]); pruefe(&mut app, "Tracker", "Neues Muster", "Muster abspielen"); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn zen_modus_zeigt_nur_flaeche_stellt_her_und_kuerzel_ist_frei() {
        use vc_ide_model::layout::Bereich;
        let (mut app, tmp) = song_app(); let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let t = texte(&mut app); assert!(t.iter().any(|x| x == "Werkzeug") && t.iter().any(|x| x.starts_with("Muster ·")) && t.iter().any(|x| x == "Protokoll"), "vorher: Seitenspalte, Liste und Schublade da");
        // Kürzel: Cmd+Umschalt+B; Cmd+B bleibt die linke Spalte
        assert_eq!(vc_gui_ui::befehle::info(Cmd::Zen).tasten, "Cmd+Umschalt+B"); assert_eq!(vc_gui_ui::befehle::info(Cmd::Seitenleiste).tasten, "Cmd+B");
        let vorher = (app.layout.offen(Bereich::Seitenleiste), app.layout.offen(Bereich::Schublade), app.layout.offen(Bereich::Panel));
        let _ = app.befehl(Cmd::Zen); assert!(app.layout.zen());
        let t = texte(&mut app);
        for weg in ["Werkzeug", "Klaviatur", "Kanäle", "Protokoll", "Dateien"] { assert!(!t.iter().any(|x| x == weg), "Zen: „{weg}“ ist ausgeblendet: {t:?}") }
        assert!(!t.iter().any(|x| x.starts_with("Muster ·")), "Zen: keine Musterliste");
        for bleibt in ["TRK", "Noten", "Instrumente", "Ablauf", "▶ Song", "Speichern"] { assert!(t.iter().any(|x| x == bleibt), "Zen: „{bleibt}“ bleibt (Kopfzeile, Leiste, Fläche): {t:?}") }
        assert!(t.iter().any(|x| x.starts_with("Muster 00 · Zeile")), "Fußzeile bleibt");
        // erneut: vorheriger Zustand; nichts davon landet in den gespeicherten Einstellungen
        let _ = app.befehl(Cmd::Zen); assert!(!app.layout.zen()); assert_eq!((app.layout.offen(Bereich::Seitenleiste), app.layout.offen(Bereich::Schublade), app.layout.offen(Bereich::Panel)), vorher);
        let t = texte(&mut app); assert!(t.iter().any(|x| x == "Werkzeug") && t.iter().any(|x| x == "Protokoll"), "wieder da");
        // gemerkt wird im Zen-Modus das Layout davor
        let _ = app.befehl(Cmd::Zen); assert!(app.einst.seitenleiste == vorher.0 && app.einst.schublade == vorher.1, "Einstellungen behalten das Layout davor");
        // das Studio wechselt die Breitenklasse nicht heimlich: in jeder Klasse nur die Fläche
        for (b, h) in [(1280, 800), (900, 700), (720, 800)] { let t: Vec<String> = knoten_sammeln(&mut app, b, h).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect(); assert!(!t.iter().any(|x| x == "Klaviatur" || x == "Werkzeug"), "{b}×{h}: Zen ohne Seitenspalte"); }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn takt_anzeige_in_der_statusleiste_ist_standardmaessig_aus_zeigt_zielwerte_und_verschwindet_im_schmalen_fenster() {
        let (mut app, tmp) = song_app(); let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App, b: u32, h: u32| -> Vec<String> { knoten_sammeln(app, b, h).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        assert!(!app.einst.takt_anzeige, "Standard: aus"); assert!(!texte(&mut app, 1280, 800).iter().any(|x| x.contains("MHz")), "aus: nichts");
        let _ = app.update(Msg::SetTakt(true)); assert!(app.einst.takt_anzeige);
        let t = texte(&mut app, 1280, 800); assert!(t.iter().any(|x| x.starts_with("6510 · 0,985 MHz · PAL") && x.contains("312 Zeilen") && x.contains("63 Zyklen/Zeile")), "C64 PAL: {:?}", t.iter().filter(|x| x.contains("MHz")).collect::<Vec<_>>());
        assert!(!texte(&mut app, 720, 800).iter().any(|x| x.contains("MHz")), "schmales Fenster: ausgeblendet");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn pfadfelder_haben_waehlen_und_tragen_den_pfad_ueber_denselben_weg_ein() {
        use pfadfeld::{PfadArt, Setzer}; use std::sync::Arc;
        let (mut app, tmp) = disk_app(); app.oeffne("spiel.d64"); let _ = simuliere(&mut app, &[]); let rel = "spiel.d64";
        let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect(); assert!(t.iter().any(|x| x == "Wählen …"), "Disk: Zielordner mit „Wählen …“: {t:?}");
        let r = rel.to_string(); let setzer = Setzer(Arc::new(move |z| Msg::DiskAkt(r.clone(), diskstudio::DAkt::Zielordner(z))));
        let _ = app.update(Msg::PfadGewaehlt(PfadArt::ProjektOrdner, setzer.clone(), Some(tmp.join("sub/x")))); assert_eq!(app.disks[rel].opts.dir, "sub/x", "Ordner im Projekt wird relativ eingetragen");
        let _ = app.update(Msg::PfadGewaehlt(PfadArt::ProjektOrdner, setzer.clone(), None)); assert_eq!(app.disks[rel].opts.dir, "sub/x", "Abbruch im Dialog ändert nichts");
        let _ = app.update(Msg::DiskPfad(rel.into(), diskstudio::DPfad::SpeichernUnter, Some(tmp.join("kopie")))); assert!(tmp.join("kopie.d64").is_file(), "Endungs-Automatik: Speichern unter ohne Endung ergänzt .d64");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn markdown_dateien_zeigen_die_vorschau_mit_umschalter_quelltext_und_die_lizenzhinweise_in_bloecken() {
        let (mut app, tmp) = studio_app(); std::fs::write(tmp.join("notiz.md"), "# Titel der Notiz\n\nEin Absatz mit **fett** und `code`.\n\n- erster Punkt\n- [x] erledigt\n\n| a | b |\n|---|---|\n| 1 | 2 |\n").unwrap(); std::fs::write(tmp.join("leer.md"), "").unwrap(); app.projekt_neu_laden();
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        app.oeffne("notiz.md"); let _ = simuliere(&mut app, &[]); let t = texte(&mut app);
        assert!(t.iter().any(|x| x == "Markdown-Vorschau") && t.iter().any(|x| x == "Quelltext") && !t.iter().any(|x| x.contains("**fett**") || x.starts_with("# ")), "Vorschau statt Quelltext (kein ** und #): {t:?}");
        assert!(t.iter().any(|x| x == "☑") && t.iter().any(|x| x == "•"), "Aufgabe und Aufzählung als Zeichen");
        let _ = app.befehl(Cmd::MdUmschalten); let t = texte(&mut app); assert!(t.iter().any(|x| x == "Markdown-Quelltext") && t.iter().any(|x| x == "Vorschau"), "Quelltext mit Weg zurück: {t:?}");
        let _ = app.update(Msg::MdUmschalten("notiz.md".into())); assert!(app.md_vorschau("notiz.md"));
        app.oeffne("leer.md"); let _ = simuliere(&mut app, &[]); assert!(!app.md_vorschau("leer.md"), "leere Dateien öffnen im Editor (zum Schreiben)");
        // Lizenzhinweise: große Datei in Blöcken, geparst nur einmal
        let doc = app.md_doc(infodlg::HINWEISE); assert!(doc.nodes.len() > mdansicht::BLOECKE, "die Hinweise sind länger als ein Block-Satz: {}", doc.nodes.len()); let a = std::rc::Rc::as_ptr(&app.md_doc(infodlg::HINWEISE)); assert_eq!(a, std::rc::Rc::as_ptr(&doc), "aus dem Zwischenspeicher");
        let _ = app.update(Msg::MdMehr); assert_eq!(app.md_grenze, 2 * mdansicht::BLOECKE);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn dateibaum_laesst_sich_mit_der_tastatur_bedienen() {
        use iced::keyboard::{key::Named, Key, Modifiers};
        let (mut app, _) = App::neu(); let tmp = std::env::temp_dir().join(format!("vcgui-baumtaste-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(tmp.join("lib")).unwrap(); std::fs::create_dir_all(tmp.join("zeug")).unwrap();
        std::fs::write(tmp.join("main.asm"), "nop\n").unwrap(); std::fs::write(tmp.join("lib/a.inc"), "x=1\n").unwrap(); std::fs::write(tmp.join("lib/b.inc"), "y=2\n").unwrap(); std::fs::write(tmp.join("zeug/z.txt"), "z\n").unwrap();
        app.papierkorb = false; app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.editoren.clear(); einstellungen::test_konfig(tmp.join("konf")); app.fokus_bereich = Bereich::Seitenleiste; app.seite = Seite::Dateien;
        let taste = |app: &mut App, n: Named| { let _ = app.update(Msg::Taste(Key::Named(n), Modifiers::empty())); };
        taste(&mut app, Named::ArrowDown); assert_eq!(app.baum_sel.as_deref(), Some("lib"), "erste Zeile (Ordner kommen zuerst)");
        taste(&mut app, Named::ArrowDown); assert_eq!(app.baum_sel.as_deref(), Some("lib/a.inc"), "↓ geht durch die sichtbaren Zeilen (Ordner sind offen)");
        taste(&mut app, Named::ArrowUp); assert_eq!(app.baum_sel.as_deref(), Some("lib"));
        taste(&mut app, Named::ArrowRight); assert_eq!(app.baum_sel.as_deref(), Some("lib/a.inc"), "→ auf offenem Ordner: erstes Kind");
        taste(&mut app, Named::ArrowLeft); assert_eq!(app.baum_sel.as_deref(), Some("lib"), "← auf einer Datei: zum Ordner");
        taste(&mut app, Named::ArrowLeft); assert!(app.baum_zu.contains("lib"), "← auf offenem Ordner: zuklappen"); taste(&mut app, Named::ArrowRight); assert!(!app.baum_zu.contains("lib"), "→ klappt wieder auf");
        taste(&mut app, Named::End); assert_eq!(app.baum_sel.as_deref(), Some("main.asm")); taste(&mut app, Named::Home); assert_eq!(app.baum_sel.as_deref(), Some("lib"));
        taste(&mut app, Named::End); taste(&mut app, Named::Enter); assert_eq!(app.name().as_deref(), Some("main.asm"), "Enter öffnet die Datei");
        // F2 benennt die gewählte Zeile um, Entf löscht (Dialog)
        app.fokus_bereich = Bereich::Seitenleiste; app.baum_sel = Some("lib/b.inc".into()); taste(&mut app, Named::F2); assert!(matches!(app.baum.as_ref().map(|b| &b.art), Some(BaumArt::Umbenennen(n)) if n == "lib/b.inc")); let _ = app.update(Msg::BaumAbbruch);
        taste(&mut app, Named::Delete); assert!(matches!(app.baum.as_ref().map(|b| &b.art), Some(BaumArt::Loeschen(n)) if n == "lib/b.inc")); let _ = app.update(Msg::BaumAbbruch);
        // Fokus im Dokument: Pfeile gehören dem Editor, der Baum bleibt unberührt
        app.fokus_bereich = Bereich::Dokument; app.baum_sel = Some("lib".into()); taste(&mut app, Named::ArrowDown); assert_eq!(app.baum_sel.as_deref(), Some("lib"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn basic_wird_in_assembler_umgewandelt_ohne_etwas_zu_ueberschreiben() {
        let tmp = kopie_von_beispiel("examples/c64/basic_native"); let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.editoren.clear(); einstellungen::test_konfig(tmp.join("konf"));
        let bas = std::fs::read_dir(&tmp).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).find(|n| n.ends_with(".bas")).expect("Beispiel hat eine .bas-Datei");
        let _ = app.befehl(Cmd::BasicNachAsm); assert!(app.log.last().is_some_and(|l| l.contains("erst eine BASIC-Datei")), "ohne geöffnete BASIC-Datei: Hinweis");
        app.oeffne(&bas); let _ = app.befehl(Cmd::BasicNachAsm);
        let stamm = bas.trim_end_matches(".bas").to_string(); let ziel = format!("{stamm}_asm.asm");
        assert!(tmp.join(&ziel).is_file() && std::fs::read_to_string(tmp.join(&ziel)).unwrap().len() > 100, "Assembler-Datei entstanden: {:?}", app.log.last());
        assert!(tmp.join(format!("{stamm}_asm.b2a.json")).is_file(), "Abbildung der Zeilen daneben"); assert_eq!(app.name().as_deref(), Some(ziel.as_str()), "die neue Datei öffnet");
        let erste = std::fs::read_to_string(tmp.join(&ziel)).unwrap();
        app.oeffne(&bas); let _ = app.befehl(Cmd::BasicNachAsm); assert!(tmp.join(format!("{stamm}_asm-2.asm")).is_file() && std::fs::read_to_string(tmp.join(&ziel)).unwrap() == erste, "zweiter Lauf: neue Nummer, die erste Datei bleibt");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn zahlenwerkzeuge_rechnen_zeigen_alle_schreibweisen_schalten_bits_und_setzen_in_den_editor_ein() {
        use zahlen::ZAkt;
        let (mut app, tmp) = song_app(); app.oeffne("music_demo.asm"); let _ = simuliere(&mut app, &[]);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let _ = app.befehl(Cmd::Zahlenwerkzeuge); assert!(app.zahlen.is_some()); let t = texte(&mut app); assert!(t.iter().any(|x| x == "Zahlenwerkzeuge") && t.iter().any(|x| x == "$D020") && t.iter().any(|x| x == "53280"), "Vorgabe $D020 in allen Schreibweisen: {t:?}");
        let _ = app.update(Msg::Zahl(ZAkt::Text("$D000+$20".into()))); let t = texte(&mut app); assert!(t.iter().any(|x| x == "$D020"), "Ausdruck gerechnet");
        let _ = app.update(Msg::Zahl(ZAkt::Text("%00000101".into()))); let _ = app.update(Msg::Zahl(ZAkt::Bit(1))); assert_eq!(zahlen::werte(&app.zahlen.as_ref().unwrap().text), Ok(7), "Bit 1 geschaltet: 5 → 7");
        let t = texte(&mut app); assert!(t.iter().any(|x| x == "%00000111") && t.iter().any(|x| x == "Bit-Editor"), "{t:?}");
        let _ = app.update(Msg::Zahl(ZAkt::Text("1+".into()))); let t = texte(&mut app); assert!(t.iter().any(|x| x.contains("Zahl erwartet")), "Fehler sichtbar, kein Absturz");
        let _ = app.update(Msg::Zahl(ZAkt::Text("$A9".into()))); let vor = app.editoren["music_demo.asm"].text(); let _ = app.update(Msg::Zahl(ZAkt::Einsetzen("hex")));
        assert!(app.zahlen.is_none() && app.editoren["music_demo.asm"].text().len() == vor.len() + 3 && app.editoren["music_demo.asm"].text().contains("$A9"), "Einsetzen schreibt „$A9“ an die Schreibmarke und schließt");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn startbild_ist_standard_an_zeigt_logo_und_version_und_schliesst_per_klick_taste_und_einstellung() {
        use iced::keyboard::{Key, Modifiers};
        let (mut app, tmp) = studio_app(); let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        assert!(app.einst.splash, "Vorgabe: an"); assert!(!texte(&mut app).iter().any(|x| x.starts_with("Version ")), "im Test startet es nicht von selbst");
        app.splash = true; let t = texte(&mut app); assert!(t.iter().any(|x| x == "VintageCoder") && t.iter().any(|x| x.starts_with("Version ")) && t.iter().any(|x| x == "Alpha-Version"), "Logo, Version und Hinweis: {t:?}");
        let _ = app.update(Msg::SplashZu); assert!(!app.splash); app.splash = true; let _ = app.update(Msg::Taste(Key::Character("x".into()), Modifiers::empty())); assert!(!app.splash, "jede Taste schließt, ohne etwas zu tun");
        let _ = app.update(Msg::SetSplash(false)); assert!(!app.einst.splash, "abschaltbar");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn konfiguration_laesst_sich_exportieren_importieren_mit_rueckfrage_und_zuruecksetzen() {
        use konfig::KArt;
        let (mut app, tmp) = studio_app(); einstellungen::test_konfig(tmp.join("konf")); let datei = tmp.join("sicherung");
        app.satz.entwurf.projects_dir = "/tmp/meine-projekte".into(); app.satz.entwurf.machines.push(vc_ide::settings::Machine { id: "m1".into(), host: "192.0.2.5".into(), password: "pw1".into(), ..Default::default() }); app.satz.speichern();
        let texte = |app: &mut App| -> Vec<String> { let _ = app.befehl(Cmd::Einstellungen); knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let t = texte(&mut app); assert!(t.iter().any(|x| x == "Exportieren …") && t.iter().any(|x| x == "Importieren …") && t.iter().any(|x| x == "Zurücksetzen …"), "Knöpfe in Einstellungen → Allgemein: {t:?}");
        // Export: ohne Passwort, Endung ergänzt
        let _ = app.update(Msg::KonfigPfad(KArt::Export, Some(datei.clone()))); let f = tmp.join("sicherung.json"); assert!(f.is_file(), "Endungs-Automatik: .json");
        let text = std::fs::read_to_string(&f).unwrap(); assert!(text.contains("meine-projekte") && !text.contains("pw1"), "Export ohne Passwörter: {text}");
        // ändern, dann Import: Rückfrage mit Zusammenfassung, dann übernommen, Passwort der bekannten Maschine bleibt
        app.satz.entwurf.projects_dir = "/anderswo".into(); app.satz.speichern();
        let _ = app.update(Msg::KonfigPfad(KArt::Import, Some(f.clone()))); assert!(matches!(&app.rueckfrage, Some(Rueckfrage::KonfigImport(_, z)) if z.contains("1 Maschine")), "Rückfrage mit Zusammenfassung: {:?}", app.rueckfrage);
        let _ = app.update(Msg::RueckfrageJa); assert_eq!(app.satz.entwurf.projects_dir, "/tmp/meine-projekte"); assert_eq!(app.satz.entwurf.machines[0].password, "pw1", "Passwort der bekannten Maschine bleibt");
        // Abbruch der Rückfrage ändert nichts; Zurücksetzen fragt und stellt die Vorgabe her
        let _ = app.update(Msg::KonfigResetFrage); assert!(matches!(app.rueckfrage, Some(Rueckfrage::KonfigReset))); let _ = app.update(Msg::RueckfrageNein); assert_eq!(app.satz.entwurf.projects_dir, "/tmp/meine-projekte");
        let _ = app.update(Msg::KonfigResetFrage); let _ = app.update(Msg::RueckfrageJa); assert!(app.satz.entwurf.projects_dir.is_empty() && app.satz.entwurf.machines.is_empty(), "zurückgesetzt");
        // kaputte Datei: Meldung, keine Rückfrage
        std::fs::write(tmp.join("kaputt.json"), "{ nein").unwrap(); let _ = app.update(Msg::KonfigPfad(KArt::Import, Some(tmp.join("kaputt.json")))); assert!(app.rueckfrage.is_none() && app.satz.meldung.as_ref().is_some_and(|(ok, m)| !ok && m.starts_with("Import")), "{:?}", app.satz.meldung);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn eigene_standardzeichensaetze_sind_vorlage_mit_herkunftskarte_und_werden_als_kopie_geladen() {
        use studio::Aktion;
        let (mut app, tmp) = studio_app(); let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::fs::copy(repo.join("examples/c64/gfx_demo/graphics/font.vcchar"), tmp.join("schrift.vcchar")).unwrap(); app.projekt_neu_laden(); app.oeffne("schrift.vcchar"); let _ = simuliere(&mut app, &[]); let rel = "schrift.vcchar";
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let v = app.studios[rel].doc.vorlagen(&app.data); assert!(v.iter().any(|(r, l, fehlt)| r == "std:vc-c64-upper" && l.starts_with("Standard (eigen):") && !fehlt), "eigene Sätze in der Vorlagenliste: {v:?}");
        // laden: Kopie, kein Verweis in der Datei, origin da
        let _ = app.update(Msg::StudioAkt(rel.into(), Aktion::Vorlage("std:vc-c64-upper".into()))); let d = &app.studios[rel].doc;
        assert_eq!(d.vorlage_aktuell(), "std:vc-c64-upper"); assert!(d.media["base"].is_null() || d.media["base"] == "", "kein Verweis (rom:/std:) in der Datei"); assert_eq!(d.media["origin"]["source"], "vintagecoder-standard");
        let h = d.herkunft(&app.data).expect("Herkunft"); assert_eq!(h.0, "EIGEN", "unverändert: EIGEN"); assert!(h.1.iter().any(|z| z.contains("Lizenz: CC0-1.0")) && h.1.iter().any(|z| z.contains("Freigabe: erteilt") && z.contains("nicht bestanden")), "{:?}", h.1);
        let t = texte(&mut app); assert!(t.iter().any(|x| x == "Herkunft") && t.iter().any(|x| x == "EIGEN"), "Karte Herkunft mit Etikett als Wort: {t:?}");
        // eigene Änderung → EIGEN, GEÄNDERT
        { let d = &mut app.studios.get_mut(rel).unwrap().doc; let alt = d.frames[0][0][0]; d.setze(0, 0, if alt == 0 { 1 } else { 0 }); }
        let t = texte(&mut app); assert!(t.iter().any(|x| x == "EIGEN, GEÄNDERT"), "geändert: {t:?}");
        // Rückgängig der Vorlage
        let _ = app.befehl(Cmd::Rueckgaengig); let _ = app.befehl(Cmd::Rueckgaengig); assert!(app.studios[rel].doc.media["origin"].is_null(), "Rückgängig nimmt den eigenen Satz zurück");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn einzeltasten_der_studios_sind_mit_einem_schalter_abschaltbar_kuerzel_und_pfeile_bleiben() {
        use iced::keyboard::{key::Named, Key, Modifiers}; use vc_ide_model::pixel::Tool;
        let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite"; einstellungen::test_konfig(tmp.join("konf"));
        assert!(app.einst.studio_einzeltasten, "Vorgabe: an");
        let _ = app.update(Msg::Taste(Key::Character("e".into()), Modifiers::empty())); assert_eq!(app.studios[rel].doc.tool, Tool::Erase, "E wählt den Radierer");
        let _ = app.update(Msg::SetStudioTasten(false)); assert!(!app.einst.studio_einzeltasten);
        let _ = app.update(Msg::Taste(Key::Character("d".into()), Modifiers::empty())); assert_eq!(app.studios[rel].doc.tool, Tool::Erase, "aus: D tut nichts");
        let _ = app.update(Msg::Taste(Key::Character("1".into()), Modifiers::empty())); // Ziffern ebenso
        let _ = app.update(Msg::Taste(Key::Named(Named::ArrowRight), Modifiers::empty())); // Pfeile bleiben (Frame wechseln)
        let _ = app.befehl(Cmd::Rueckgaengig); // Kürzel/Befehle bleiben
        let t: Vec<String> = { let _ = app.update(Msg::Set(setui::SetAkt::Seite(setui::Seite::Editor))); knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() }; let _ = t;
        let _ = app.befehl(Cmd::Einstellungen); let _ = app.update(Msg::Set(setui::SetAkt::Seite(setui::Seite::Editor))); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect();
        assert!(t.iter().any(|x| x == "Einzeltasten der Studios"), "Schalter auf der Seite Editor: {t:?}");
        let _ = app.update(Msg::SetStudioTasten(true)); let _ = app.update(Msg::Taste(Key::Character("d".into()), Modifiers::empty()));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn studio_regeln_k1_befehle_kontextmenue_im_tracker_und_pfeile_leertaste_im_bank_studio() {
        use iced::keyboard::{key::Named, Key, Modifiers}; use bankstudio::BAkt;
        // (c) Studio-Aktionen sind Befehle in der Tabelle: Zoom/Einpassen im Sprite- und im Bank-Studio, Raster im Sprite-Studio
        let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite"; let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
        assert!(vc_gui_ui::befehle::BEFEHLE.iter().any(|b| b.id == "studio_einpassen") && vc_gui_ui::befehle::BEFEHLE.iter().any(|b| b.id == "studio_vergroessern"), "Befehle in der Tabelle");
        let sk = app.studios[rel].zoomansicht.zoom_wert(12.0, 21.0); let _ = app.befehl(Cmd::StudioVergroessern); assert!(app.studios[rel].zoomansicht.zoom.is_some_and(|z| z >= sk.floor()), "vergrößert");
        let _ = app.befehl(Cmd::StudioEinpassen); assert!(app.studios[rel].zoomansicht.zoom.is_none(), "Einpassen per Befehl"); let r0 = app.studios[rel].raster; let _ = app.befehl(Cmd::StudioRaster); assert_ne!(app.studios[rel].raster, r0, "Raster per Befehl");
        let _ = std::fs::remove_dir_all(&tmp);
        // (e) Bank-Studio: Pfeile bewegen die aktive Kachel, Leertaste setzt dort wie ein Klick
        let (mut app, tmp) = screen_app(); let rel = "graphics/level.vcscreen"; app.oeffne(rel); let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
        let gitter = |app: &App| app.banken[rel].doc.bank.screens[0].grid.clone(); let _ = app.update(Msg::BankAkt(rel.into(), BAkt::Kachel(1)));
        assert!(app.bank_taste(rel, &Key::Named(Named::ArrowRight), Modifiers::empty())); assert!(app.bank_taste(rel, &Key::Named(Named::ArrowDown), Modifiers::empty())); assert!(app.bank_taste(rel, &Key::Named(Named::ArrowDown), Modifiers::empty()));
        let u = app.banken[rel].ueber.expect("aktive Einheit"); let tw = app.banken[rel].doc.ts.as_ref().unwrap().tw as i32; assert_eq!(u.0 % tw, 0, "Schritt = eine Kachel: {u:?}");
        let (gx, gy) = (u.0 / tw, u.1 / tw); let vor = gitter(&app)[gy as usize][gx as usize]; assert_ne!(vor, 1, "Zelle war nicht schon Kachel 1");
        assert!(app.bank_taste(rel, &Key::Named(Named::Space), Modifiers::empty())); assert_eq!(gitter(&app)[gy as usize][gx as usize], 1, "Leertaste setzt die gewählte Kachel an der aktiven Einheit");
        let _ = std::fs::remove_dir_all(&tmp);
        // (a) Tracker: Kontextmenü der Fläche (Rechtsklick) mit Kopieren/Einfügen, Befehle wirken auf den Tracker
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]); let _ = simuliere(&mut app, &[]);
        let _ = app.update(Msg::KontextAuf(Art::Flaeche, iced::Point::new(600.0, 300.0), Some(rel.into()))); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect();
        for n in ["Kopieren", "Einfügen", "Ausschneiden"] { assert!(t.iter().any(|x| x == n), "{n} im Kontextmenü der Tracker-Fläche: {t:?}") }
        let _ = app.update(Msg::KontextZu); { let d = &mut app.songs.get_mut(rel).unwrap().doc; d.cursor = (0, 0); } let _ = app.befehl(Cmd::Kopieren); assert!(app.songs[rel].doc.zwischen.is_some(), "Kopieren (Befehl) gilt im Tracker"); { let d = &mut app.songs.get_mut(rel).unwrap().doc; d.cursor = (1, 1); } let _ = app.befehl(Cmd::Einfuegen); assert!(app.songs[rel].doc.dirty(), "Einfügen (Befehl) gilt im Tracker");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_wiedergabe_ueber_aussen_attrappe_position_mitlaufen_vorhoeren_stumm() {
        use iced::{keyboard::{key::Named, Key, Modifiers}, Point}; use songstudio::GAkt;
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]); let _ = aussen::aufgezeichnet();
        let taste = |app: &mut App, z: &str| { let (ev, key, mods) = kuerzel_ev(z); let (msgs, st) = ui_status(app, ev, Point::new(500.0, 400.0)); for m in msgs { let _ = app.update(m); } if st.iter().all(|s| *s == iced::event::Status::Ignored) { let _ = app.update(Msg::Taste(key, mods)); } };
        // Vorhören: Note eintippen → Aufruf aufgezeichnet, kein Gerät geöffnet
        taste(&mut app, "Z"); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::AudioVorhoeren { note: 37 }), "Note wird vorgehört (Attrappe)"); assert!(app.audio.fehler().is_none());
        // Leertaste: Song starten; Position läuft mit, Muster folgt der Reihenfolge
        let _ = app.update(Msg::Taste(Key::Named(Named::Space), Modifiers::empty())); assert!(app.audio.spielt() && app.audio_song.as_deref() == Some(rel) && app.songs[rel].spiel == Some((0, 0)), "{:?}", aussen::aufgezeichnet());
        let ord = app.songs[rel].doc.reihenfolge(); let row_secs = app.songs[rel].doc.song_daten(false).row_secs; let n = (row_secs * 44100.0 * 40.0) as usize; app.audio.schritt(n); let _ = app.update(Msg::SongTick);
        let (o, z) = app.songs[rel].spiel.unwrap(); assert!(z > 0 || o > 0, "Position ist weitergelaufen: {o}/{z}"); assert_eq!(app.songs[rel].doc.muster, ord[o.min(ord.len() - 1)], "das angezeigte Muster folgt der Reihenfolge");
        assert!(app.songs[rel].pegel.1 >= 0.0); let _ = simuliere(&mut app, &[]);
        // Stumm/Solo während des Spiels; Stopp per Leertaste
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Stumm(1))); assert!(app.audio.stumm_ist(1)); let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Solo(2))); assert!(app.audio.solo_ist(2));
        let _ = app.update(Msg::Taste(Key::Named(Named::Space), Modifiers::empty())); assert!(!app.audio.spielt() && app.audio_song.is_none() && app.songs[rel].spiel.is_none()); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::AudioStopp));
        // nur Muster (Umschalt+Leertaste): endet nach einem Durchlauf von selbst
        let _ = app.update(Msg::Taste(Key::Named(Named::Space), Modifiers::SHIFT)); assert!(app.audio.spielt()); app.audio.schritt((row_secs * 44100.0 * 80.0) as usize); let _ = app.update(Msg::SongTick); let _ = app.update(Msg::SongTick);
        assert!(app.audio_song.is_none() || !app.audio.spielt() || app.songs[rel].spiel.is_some(), "Zustand bleibt stimmig");
        // Projektwechsel stoppt die Wiedergabe
        let _ = app.update(Msg::SongAkt(rel.into(), GAkt::SpielSong)); assert!(app.audio.spielt()); let tmp2 = kopie_von_beispiel("examples/c64/prallball"); assert!(app.projekt_oeffnen(tmp2.clone())); assert!(!app.audio.spielt() && app.audio_song.is_none());
        let _ = (std::fs::remove_dir_all(&tmp), std::fs::remove_dir_all(&tmp2));
    }
    #[test] fn play_tracker_oeffnet_im_tracker_studio_mit_tempo_lautstaerke_wiedergabe_und_basic_export() {
        use songstudio::GAkt; use iced::keyboard::{key::Named, Key, Modifiers};
        let tmp = kopie_von_beispiel("examples/mega65/play_demo"); einstellungen::test_konfig(tmp.join("konf")); let _ = aussen::aufgezeichnet();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone())); let rel = "music/tune_play.vcplay"; app.oeffne(rel);
        assert!(app.songs.contains_key(rel) && app.songs[rel].doc.ist_play() && app.name().as_deref() == Some(rel), "PLAY-Datei öffnet im Tracker-Studio"); let _ = simuliere(&mut app, &[]);
        let (v0, t0) = (app.songs[rel].doc.lautstaerke(), app.songs[rel].doc.tempo()); let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Lautstaerke(-3))); let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Tempo(5)));
        assert_eq!((app.songs[rel].doc.lautstaerke(), app.songs[rel].doc.tempo()), (v0 - 3, t0 + 5)); assert!(app.songs[rel].doc.dirty());
        // Noten tippen und vorhören, Wiedergabe über die Attrappe
        let _ = app.update(Msg::Taste(Key::Character("x".into()), Modifiers::empty())); assert!(aussen::aufgezeichnet().iter().any(|a| matches!(a, aussen::Aufruf::AudioVorhoeren { .. })));
        let _ = app.update(Msg::Taste(Key::Named(Named::Space), Modifiers::empty())); assert!(app.audio.spielt()); app.audio.schritt(44100); let _ = app.update(Msg::SongTick); assert!(app.songs[rel].spiel.is_some()); let _ = app.update(Msg::Taste(Key::Named(Named::Space), Modifiers::empty())); assert!(!app.audio.spielt());
        // W7: Änderung während der Wiedergabe geht live an die Wiedergabe; W2: Raster scrollt mit, Handscrollen beendet das Mitlaufen
        let _ = app.update(Msg::Taste(Key::Named(Named::Space), Modifiers::empty())); assert!(app.audio.spielt()); app.songs.get_mut(rel).unwrap().ansicht = (0.0, 0.0, 600.0, 200.0);
        app.audio.schritt(2000); let _ = app.update(Msg::SongTick); let vor = aussen::aufgezeichnet().iter().filter(|a| matches!(a, aussen::Aufruf::AudioAktualisieren)).count();
        let _ = app.update(Msg::Taste(Key::Character("c".into()), Modifiers::empty())); let _ = app.update(Msg::SongTick);
        assert!(aussen::aufgezeichnet().iter().filter(|a| matches!(a, aussen::Aufruf::AudioAktualisieren)).count() > vor, "Änderung während der Wiedergabe wird übergeben"); assert!(app.audio.spielt());
        assert!(app.songs[rel].mitlaufen); let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Mitlaufen(false))); assert!(!app.songs[rel].mitlaufen);
        let _ = app.update(Msg::Taste(Key::Named(Named::Space), Modifiers::empty())); assert!(!app.audio.spielt());
        let _ = app.befehl(Cmd::Speichern); assert!(!app.songs[rel].doc.dirty());
        // Export-Dialog für den PLAY-Tracker: BASIC-Befehle
        let _ = app.befehl(Cmd::Exportieren); let d = app.export.as_ref().expect("Dialog offen"); assert_eq!(d.rel, rel); assert!(d.cache.error.is_none(), "{:?}", d.cache.error);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn tracker_konvertieren_und_rmt_import_dialog_legt_neue_datei_im_projekt_an() {
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]);
        // Quelle: Song eines anderen Systems (POKEY), außerhalb des Projekts, aus einer Umwandlung des Beispiels
        let fremd = std::env::temp_dir().join(format!("vcgui-fremd-{}", std::process::id())); let _ = std::fs::remove_dir_all(&fremd); std::fs::create_dir_all(&fremd).unwrap();
        let p = vc_ide_model::songimport::konvertieren(&tmp, &app.data, &tmp.join(rel), "atari.pokey.song", "tmp_atari", "music", 0, 30).unwrap(); std::fs::rename(tmp.join(&p.0), fremd.join("aus atari.vcsong")).unwrap();
        let _ = app.update(Msg::SongDatei(rel.into(), false, Some(fremd.join("aus atari.vcsong")))); let d = app.song_import.as_ref().expect("Dialog offen"); assert_eq!((d.name.as_str(), d.quelle.art.as_str(), d.ziel.as_str()), ("aus_atari", "POKEY (Atari)", "SID (C64)"));
        let _ = simuliere(&mut app, &[]); // Dialog zeichnet
        // ungültiger Name: Fehler bleibt im Dialog, nichts angelegt
        let _ = app.update(Msg::SongImpName("falsch name!".into())); let job = app.song_import_job().unwrap(); app.song_import_fertig(job()); assert!(app.song_import.as_ref().is_some_and(|d| !d.fehler.is_empty()), "Fehler im Dialog"); assert!(!tmp.join("music/falsch name!.vcsong").exists());
        let _ = app.update(Msg::SongImpName("aus_atari".into())); let job = app.song_import_job().unwrap(); app.song_import_fertig(job());
        assert!(app.song_import.is_none() && tmp.join("music/aus_atari.vcsong").is_file(), "{:?}", app.log.last()); assert!(app.songs.contains_key("music/aus_atari.vcsong") && app.name().as_deref() == Some("music/aus_atari.vcsong"), "neue Datei öffnet im Tracker"); assert!(app.log.iter().any(|l| l == "Angelegt: music/aus_atari.vcsong"), "{:?}", app.log);
        assert_eq!(app.songs["music/aus_atari.vcsong"].doc.kanaele, 3, "Ziel ist das SID-Profil des offenen Trackers");
        // unlesbare SID-Datei: Fehler im Protokoll, kein Dialog; Abbrechen schließt
        std::fs::write(fremd.join("kaputt.sid"), b"nein").unwrap(); let _ = app.update(Msg::SongDatei(rel.into(), false, Some(fremd.join("kaputt.sid")))); assert!(app.song_import.is_none() && app.log.last().is_some_and(|l| l.contains("kaputt.sid")), "{:?}", app.log.last());
        let _ = app.update(Msg::SongDatei(rel.into(), false, Some(fremd.join("aus atari.vcsong")))); let _ = app.update(Msg::SongImpAbbruch); assert!(app.song_import.is_none());
        // RMT: Nicht-Modul → Fehler im Dialog; nur bei POKEY-Songs bietet die Leiste den Knopf an
        let _ = app.update(Msg::SongDatei(rel.into(), true, Some(fremd.join("kaputt.sid")))); let job = app.song_import_job().unwrap(); app.song_import_fertig(job()); assert!(app.song_import.as_ref().is_some_and(|d| d.rmt && !d.fehler.is_empty()), "kein RMT-Modul: Fehler"); let _ = app.update(Msg::SongImpAbbruch);
        let _ = (std::fs::remove_dir_all(&tmp), std::fs::remove_dir_all(&fremd));
    }
    #[test] fn tracker_tasten_kollidieren_nicht_mit_der_befehlstabelle() {
        for k in songstudio::TRACKER_TASTEN.iter().chain(songstudio::TRACKER_CMD_TASTEN) { for i in befehle::BEFEHLE { assert!(!i.tasten.eq_ignore_ascii_case(k), "Tracker-Taste {k} kollidiert mit {} ({})", i.name, i.tasten) } }
        // die Noteneingabe nutzt Buchstaben und Ziffern, Oktave/Instrument/Notenende die Zeichen , . [ ] 1: keine Überschneidung zwischen Klaviertasten und diesen
        for z in [",", ".", "[", "]"] { assert!(vc_ide_model::songdoc::klaviertaste(z.chars().next().unwrap()).is_none(), "{z} ist keine Klaviertaste") } assert!(vc_ide_model::songdoc::klaviertaste('1').is_none(), "1 ist Notenende, keine Klaviertaste");
        // keine doppelten Einträge in der Tracker-Tastentabelle
        let mut v: Vec<&str> = songstudio::TRACKER_TASTEN.to_vec(); v.sort(); let n = v.len(); v.dedup(); assert_eq!(v.len(), n);
    }
    // ---------------------------------------------------------------------------------------------- Galerie und Farbprüfung
    /// Alle Ansichten und Zustände der Oberfläche: Name und Aufbau (App mit Aufräumordner)
    fn galerie_zustaende() -> Vec<(&'static str, Box<dyn Fn() -> (App, std::path::PathBuf)>)> {
        let mut v: Vec<(&'static str, Box<dyn Fn() -> (App, std::path::PathBuf)>)> = vec![];
        v.push(("leer", Box::new(|| { let tmp = std::env::temp_dir().join(format!("vcgui-gal-leer-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap(); let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); (app, tmp) })));
        v.push(("editor", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; (app, tmp) })));
        v.push(("editor-suche", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.befehl(Cmd::Suchen); (app, tmp) })));
        v.push(("baum-menue", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.update(Msg::KontextAuf(Art::BaumFlaeche, iced::Point::new(120.0, 400.0), Some(String::new()))); (app, tmp) })));
        v.push(("befehlspalette", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.befehl(Cmd::Palette); (app, tmp) })));
        v.push(("menueleiste", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; app.menue_test = true; app.menue_oeffnen("Datei"); (app, tmp) })));
        v.push(("menueleiste-schmal", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; app.menue_test = true; app.fenster = iced::Size::new(520.0, 800.0); app.menue_oeffnen("*"); (app, tmp) })));
        v.push(("zen", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.befehl(Cmd::Zen); (app, tmp) })));
        v.push(("takt", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; app.einst.takt_anzeige = true; (app, tmp) })));
        v.push(("einstellungen", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.befehl(Cmd::Einstellungen); (app, tmp) })));
        v.push(("einstellungen-roms", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let roms = tmp.join("roms-test"); std::fs::create_dir_all(roms.join("c64")).unwrap(); std::fs::write(roms.join("c64/kernal.bin"), vec![1u8; 8192]).unwrap(); app.satz.roms.test_ordner = Some(roms); let _ = app.befehl(Cmd::Einstellungen); app.satz.seite = setui::Seite::Roms; let st = vc_roms::RomStore::new(app.rom_env()); app.satz.roms.stand = Some(std::sync::Arc::new(st)); (app, tmp) })));
        v.push(("startseite", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; app.tabs.clear(); app.aktiv = 0; (app, tmp) })));
        let leer_app = |name: &'static str| -> (App, std::path::PathBuf) { let tmp = std::env::temp_dir().join(format!("vcgui-gal-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap(); let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.start.recent.clear(); (app, tmp) };
        v.push(("startseite-leer", Box::new(move || leer_app("sl"))));
        v.push(("startseite-zuletzt", Box::new(move || { let (mut app, tmp) = leer_app("sz"); let info = vc_ide_model::recent::Info { systems: vec![("c64".into(), "pal".into())], langs: vec!["vcasm".into()], emulator: Some("VICE x64sc".into()), ..Default::default() }; app.start.recent = vec![
            vc_ide_model::recent::Recent { path: "/tmp/vc-handbuch/Prallball".into(), name: "Prallball".into(), pinned: true, last: vc_ide_model::recent::now() - 3600, missing: false, info: info.clone() },
            vc_ide_model::recent::Recent { path: "/tmp/vc-handbuch/Plasma".into(), name: "Plasma".into(), pinned: false, last: vc_ide_model::recent::now() - 86400 * 3, missing: false, info: vc_ide_model::recent::Info { systems: vec![("mega65".into(), String::new())], langs: vec!["vcasm".into()], ..Default::default() } }]; (app, tmp) })));
        v.push(("startseite-ordner-fehlt", Box::new(move || { let (mut app, tmp) = leer_app("sf"); app.start.recent = vec![vc_ide_model::recent::Recent { path: "/tmp/vc-handbuch/Verschoben".into(), name: "Verschoben".into(), pinned: false, last: vc_ide_model::recent::now() - 7200, missing: true, info: Default::default() }]; (app, tmp) })));
        v.push(("editor-crlf-latin1", Box::new(|| { let (mut app, tmp) = studio_app(); std::fs::write(tmp.join("alt.asm"), b"; \xe4\r\nstart: nop\r\n").unwrap(); app.projekt_neu_laden(); app.oeffne("alt.asm"); (app, tmp) })));
        v.push(("dialog-speichern-unter", Box::new(|| { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::NeuerReiter); app.ed().unwrap().insert("lda #1"); let _ = app.befehl(Cmd::Speichern); (app, tmp) })));
        v.push(("dialog-wiederherstellung", Box::new(|| { let (mut app, tmp) = studio_app(); app.scratch_rest = Some(vec![vc_ide_model::scratchsave::Entry { name: "Unbenannt 2".into(), text: "lda #1\nsta $d020".into(), note: false, saved_at: vc_ide_model::scratchsave::now_secs() - 600 }, vc_ide_model::scratchsave::Entry { name: "Notiz 3".into(), text: "# Idee".into(), note: true, saved_at: vc_ide_model::scratchsave::now_secs() - 7200 }]); (app, tmp) })));
        v.push(("dialog-beenden", Box::new(|| { let (mut app, tmp) = studio_app(); std::fs::write(tmp.join("a.asm"), "nop\n").unwrap(); app.projekt_neu_laden(); app.oeffne("a.asm"); app.ed().unwrap().insert("x"); let _ = app.befehl(Cmd::NeuerReiter); app.ed().unwrap().insert("lda #1"); let _ = app.befehl(Cmd::Beenden); (app, tmp) })));
        v.push(("dialog-neues-projekt", Box::new(|| { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::ProjektNeu); (app, tmp) })));
        v.push(("dialog-projekt-kopieren", Box::new(|| { let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); let _ = app.befehl(Cmd::ProjektKopieren); (app, tmp) })));
        v.push(("projekt-einstellungen", Box::new(|| { let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); let _ = app.befehl(Cmd::ProjektEinstellungen); (app, tmp) })));
        v.push(("projekt-einstellungen-diskette", Box::new(|| { let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); let _ = app.befehl(Cmd::ProjektEinstellungen); let _ = app.update(Msg::ProjEinst(projeinst::PeAkt::DiskAn(true))); let _ = app.update(Msg::ProjEinst(projeinst::PeAkt::Reiter(projeinst::PeReiter::Diskette))); (app, tmp) })));
        v.push(("baum-markierungen", Box::new(|| { let (mut app, tmp) = screen_app(); for f in ["gfx_demo.asm", "lib/gfx.inc", "graphics/level.vcscreen"] { app.oeffne(f) } let _ = app.update(Msg::BaumKlapp("lib".into())); app.editoren.get_mut("gfx_demo.asm").map(|e| e.dirty = true); let _ = app.update(Msg::OeffneDatei("graphics/title.vcscreen".into())); (app, tmp) })));
        v.push(("assistent-willkommen", Box::new(|| { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Assistent); (app, tmp) })));
        v.push(("assistent-zusammenfassung", Box::new(|| { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Assistent); let _ = app.update(Msg::Wiz(assistent::WizAkt::Weg(vc_ide::setup::Mode::Quick))); let _ = app.update(Msg::Wiz(assistent::WizAkt::System("mega65".into(), true))); let _ = app.update(Msg::Wiz(assistent::WizAkt::Springe(1))); for _ in 0..4 { let _ = app.update(Msg::Wiz(assistent::WizAkt::Weiter)); } (app, tmp) })));
        v.push(("studio-tracker-12-kanaele", Box::new(|| { let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/mega65/music_demo"); einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("music/tune4.vcsong"); (app, tmp) })));
        v.push(("studio-tracker-12-kanaele-ausgeblendet", Box::new(|| { let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/mega65/music_demo"); einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("music/tune4.vcsong"); for c in 6..12 { let _ = app.update(Msg::SongAkt("music/tune4.vcsong".into(), songstudio::GAkt::Sichtbar(c))); } (app, tmp) })));
        v.push(("dialog-ueber", Box::new(|| { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Ueber); (app, tmp) })));
        v.push(("dialog-feedback", Box::new(|| { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Feedback); (app, tmp) })));
        v.push(("einstellungen-ziele", Box::new(|| { let (mut app, tmp) = ziele_app(); app.einst.zoom = 1.0; (app, tmp) })));
        for (n, s) in [("einstellungen-toolchain", setui::Seite::Toolchain), ("einstellungen-kuerzel", setui::Seite::Kuerzel), ("einstellungen-editor", setui::Seite::Editor)] { v.push((n, Box::new(move || { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.befehl(Cmd::Einstellungen); app.satz.seite = s; (app, tmp) }))); }
        v.push(("hilfe-panel", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.update(Msg::PanelWahl(PanelSeite::Hilfe)); app.layout.aufklappen(Bereich::Panel); (app, tmp) })));
        v.push(("karte", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.update(Msg::PanelWahl(PanelSeite::Karte)); app.layout.aufklappen(Bereich::Panel); (app, tmp) })));
        v.push(("debug-angehalten", Box::new(|| debug_app())));
        v.push(("debug-panel", Box::new(|| { let (mut app, tmp) = debug_app(); let _ = app.update(Msg::DebugTick); app.panel_seite = PanelSeite::Debug; app.layout.aufklappen(Bereich::Panel); (app, tmp) })));
        v.push(("studio-sprite", Box::new(|| studio_app())));
        v.push(("studio-pmg", Box::new(|| { let tmp = kopie_von_beispiel("examples/atari/pmg_ship"); einstellungen::test_konfig(tmp.join("konf")); let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone())); app.oeffne("graphics/ship.vcpmg"); (app, tmp) })));
        v.push(("studio-zeichensatz", Box::new(|| { let (mut app, tmp) = screen_app(); app.oeffne("graphics/font.vcchar"); (app, tmp) })));
        v.push(("studio-palette", Box::new(|| palette_app())));
        v.push(("studio-disk", Box::new(|| disk_app())));
        v.push(("studio-bildschirm", Box::new(|| screen_app())));
        v.push(("studio-kachelsatz", Box::new(|| { let (mut app, tmp) = screen_app(); app.oeffne("graphics/level_tiles.vctiles"); (app, tmp) })));
        v.push(("studio-bank", Box::new(|| { let (mut app, tmp) = screen_app(); app.oeffne("graphics/level.vcscreen"); (app, tmp) })));
        v.push(("studio-tracker", Box::new(|| song_app())));
        v.push(("studio-disk-kette", Box::new(|| { let (mut app, tmp) = disk_app(); let mut d = vc_disk::D64::blank("KETTE", "01").unwrap(); d.add_prg("GROSS", &vec![7u8; 6000]).unwrap(); d.add_prg("KLEIN", &vec![1u8; 900]).unwrap(); std::fs::write(tmp.join("kette.d64"), d.to_bytes()).unwrap(); app.projekt_neu_laden(); app.oeffne("kette.d64"); let _ = app.update(Msg::DiskAkt("kette.d64".into(), diskstudio::DAkt::Datei(0))); (app, tmp) })));
        v.push(("studio-tracker-klaviatur-spielt", Box::new(|| { let (mut app, t) = song_app(); let rel = app.name().unwrap(); app.songs.get_mut(&rel).unwrap().klingt = vec![49, 53, 56, 70]; app.einst.tastaturlayout = einstellungen::Tastaturlayout::Qwertz; (app, t) })));
        v.push(("studio-play", Box::new(|| { let tmp = kopie_von_beispiel("examples/mega65/play_demo"); einstellungen::test_konfig(tmp.join("konf")); let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone())); app.oeffne("music/tune_play.vcplay"); (app, tmp) })));
        v.push(("dialog-export", Box::new(|| { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Exportieren); (app, tmp) })));
        v.push(("dialog-import", Box::new(|| { let (mut app, tmp) = studio_app(); let ext = tmp.join("ext"); std::fs::create_dir_all(&ext).unwrap(); std::fs::write(ext.join("neu.asm"), "nop\n").unwrap(); std::fs::write(ext.join("hero.vcsprite"), "{}").unwrap(); let _ = app.update(Msg::Abgelegt(ext.join("neu.asm"))); let _ = app.update(Msg::Abgelegt(ext.join("hero.vcsprite"))); (app, tmp) })));
        v.push(("dialog-neue-datei", Box::new(|| { let (mut app, tmp) = debug_app(); app.dbg = None; let _ = app.update(Msg::BaumNeu(String::new(), NeuArt::Asm)); (app, tmp) })));
        v.push(("dialog-song-import", Box::new(|| { let (mut app, tmp) = song_app(); let fremd = tmp.join("fremd.vcsong"); let p = vc_ide_model::songimport::konvertieren(&tmp, &app.data, &tmp.join("music/tune.vcsong"), "atari.pokey.song", "f", "music", 0, 30).unwrap(); std::fs::rename(tmp.join(p.0), &fremd).unwrap(); app.song_import_oeffnen("music/tune.vcsong", false, fremd); (app, tmp) })));
        v.push(("geteilt", Box::new(|| { let (mut app, tmp) = geteilt_app(); let _ = app.befehl(Cmd::Teilen); (app, tmp) })));
        v
    }
    /// Galerie: alle Ansichten in Hell/Dunkel und drei Fenstergrößen nach `build/galerie/` (mit `index.html`): `cargo test -p vc-gui galerie -- --ignored`
    /// Bilder der drei Tracker-Modi (hell und dunkel, 1280×800) für den Vergleich mit den Mockups nach `build/vergleich/`; Namenszusatz aus `VC_VERGLEICH` (vorher/nachher)
    #[test] #[ignore] fn play_schmal_foto() { let z = galerie_zustaende(); let (_, bauen) = z.iter().find(|(n, _)| *n == "studio-play").unwrap(); let (mut app, tmp) = bauen(); app.einst.zoom = 1.0; foto_gross(&mut app, &[], "/tmp/vc-play-schmal.png", 720, 800); let _ = std::fs::remove_dir_all(&tmp); }
    /// Bilder beliebiger Galerie-Zustände (hell und dunkel, 1280×800) nach `build/vergleich/`: `VC_ZUSTAENDE=studio-sprite,studio-disk VC_VERGLEICH=ist`
    #[test] #[ignore] fn studio_vergleich() {
        let aus = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/vergleich"); std::fs::create_dir_all(&aus).unwrap();
        let zusatz = std::env::var("VC_VERGLEICH").unwrap_or_else(|_| "ist".into()); let wahl = std::env::var("VC_ZUSTAENDE").unwrap_or_default();
        let gr: (u32, u32) = std::env::var("VC_GROESSE").ok().and_then(|g| g.split_once('x').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))).unwrap_or((1280, 800));
        for (name, bauen) in galerie_zustaende() { if !wahl.split(',').any(|w| w == name) { continue }
            let (mut app, tmp) = bauen(); app.einst.zoom = 1.0; if let Ok(n) = std::env::var("VC_PROJEKT") { app.proj.name = n }
            for (schema, sn) in [(Schema::Hell, "hell"), (Schema::Dunkel, "dunkel"), (Schema::Kontrast, "kontrast")] { app.einst.schema = schema; foto_gross(&mut app, &[], aus.join(format!("{zusatz}-{name}-{sn}{}.png", if gr == (1280, 800) { String::new() } else { format!("-{}x{}", gr.0, gr.1) })).to_str().unwrap(), gr.0, gr.1) }
            let _ = std::fs::remove_dir_all(&tmp); }
    }
    #[test] #[ignore] fn tracker_vergleich() {
        let aus = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/vergleich"); std::fs::create_dir_all(&aus).unwrap(); let zusatz = std::env::var("VC_VERGLEICH").unwrap_or_else(|_| "ist".into());
        for (modus, m) in [("noten", songstudio::SongModus::Noten), ("instrumente", songstudio::SongModus::Instrumente), ("ablauf", songstudio::SongModus::Ablauf)] {
            let (mut app, tmp) = song_app(); app.einst.zoom = 1.0; let rel = app.name().unwrap(); let _ = app.update(Msg::SongAkt(rel, songstudio::GAkt::Modus(m)));
            for (schema, sn) in [(Schema::Hell, "hell"), (Schema::Dunkel, "dunkel")] { app.einst.schema = schema; foto_gross(&mut app, &[], aus.join(format!("{zusatz}-tracker-{modus}-{sn}.png")).to_str().unwrap(), 1280, 800) }
            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
    include!("handbuchbilder.rs");
    /// Bilder für Website und Handbuch aus dem echten Beispielprojekt (Hugo & Gumbo, C64) in 1600×1000, Dunkel und Hell, nach `build/web/`:
    /// `cargo test -p vc-gui website_bilder -- --ignored` (nur Auswahl: `VC_ZUSTAENDE=hero,zen`)
    #[test] #[ignore] fn website_bilder() {
        let aus = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/web"); std::fs::create_dir_all(&aus).unwrap();
        let wahl = std::env::var("VC_ZUSTAENDE").unwrap_or_default();
        // Nur für das Bild: die Datei im Editor auf die ersten Zeilen kürzen (nichts wird gespeichert), damit keine halbe Zeile unten ins Fenster ragt
        let kurz = |a: &mut App| { if let Some(e) = a.ed() { e.lines.truncate(30); e.version += 1 } };
        let szenen: Vec<(&str, Box<dyn Fn(&mut App)>)> = vec![
            ("hero", Box::new(|a| { a.oeffne("jumprun.asm"); kurz(a); })),
            ("zen", Box::new(|a| { a.oeffne("jumprun.asm"); kurz(a); let _ = a.befehl(Cmd::Zen); })),
            ("takt", Box::new(|a| { a.oeffne("jumprun.asm"); kurz(a); a.einst.takt_anzeige = true; })),
            ("befehlspalette", Box::new(|a| { a.oeffne("jumprun.asm"); kurz(a); let _ = a.befehl(Cmd::Palette); a.palette.frage = "bauen".into(); })),
            ("dateibaum", Box::new(|a| { a.oeffne("jumprun.asm"); kurz(a); let _ = a.update(Msg::KontextAuf(Art::BaumFlaeche, iced::Point::new(120.0, 330.0), Some(String::new()))); })),
            ("studio-sprite", Box::new(|a| { a.oeffne("graphics/hero_base.vcsprite"); })),
            ("studio-zeichensatz", Box::new(|a| { a.oeffne("graphics/chars.vcchar"); let n = a.name().unwrap(); a.studio_aktion(&n, studio::Aktion::Frame(1)); })),
            ("studio-kacheln", Box::new(|a| { a.oeffne("graphics/hg.vctiles"); })),
            ("studio-bank", Box::new(|a| { a.oeffne("graphics/levels.vcscreen"); })),
            ("studio-tracker", Box::new(|a| { a.oeffne("music/tune.vcsong"); })),
            ("studio-tracker-instrumente", Box::new(|a| { a.oeffne("music/tune.vcsong"); let rel = a.name().unwrap(); let _ = a.update(Msg::SongAkt(rel, songstudio::GAkt::Modus(songstudio::SongModus::Instrumente))); })),
            ("studio-bildschirm", Box::new(|a| { a.oeffne("graphics/title.vcscreen"); })),
            ("dialog-export", Box::new(|a| { a.oeffne("graphics/hero_base.vcsprite"); let _ = a.befehl(Cmd::Exportieren); })),
            ("zahlenwerkzeuge", Box::new(|a| { a.oeffne("jumprun.asm"); kurz(a); let _ = a.befehl(Cmd::Zahlenwerkzeuge); })),
        ];
        for (name, richte) in &szenen {
            if !wahl.is_empty() && !wahl.split(',').any(|w| w == *name) { continue }
            let roh = kopie_von_beispiel("examples/c64/jumprun"); let tmp = roh.parent().unwrap().join(format!("web-{}", std::process::id())).join("Hugo-Gumbo"); let _ = std::fs::remove_dir_all(tmp.parent().unwrap()); std::fs::create_dir_all(tmp.parent().unwrap()).unwrap(); std::fs::rename(&roh, &tmp).unwrap(); einstellungen::test_konfig(tmp.join("konf"));
            let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.einst.zoom = 1.0; app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false;
            richte(&mut app); app.proj.name = "Hugo & Gumbo".into();
            for (schema, sn) in [(Schema::Dunkel, "dunkel"), (Schema::Hell, "hell")] { app.einst.schema = schema; foto_gross(&mut app, &[], aus.join(format!("{name}-{sn}.png")).to_str().unwrap(), 1600, 1007) }
            let _ = std::fs::remove_dir_all(tmp.parent().unwrap());
        }
    }
    #[test] #[ignore] fn galerie() {
        let aus = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/galerie"); let _ = std::fs::remove_dir_all(&aus); std::fs::create_dir_all(&aus).unwrap();
        let groessen = [(1280u32, 800u32), (900, 700), (720, 800)]; let mut html = String::from("<!doctype html><meta charset=utf-8><title>VintageCoder Galerie</title><style>body{font:14px system-ui;background:#222;color:#ddd;margin:16px}h2{margin:24px 0 6px}.z{display:flex;flex-wrap:wrap;gap:8px}figure{margin:0}img{width:320px;border:1px solid #555;display:block}figcaption{font-size:11px;color:#999}</style><h1>Galerie</h1>");
        let (mut n, mut fehler) = (0, vec![]);
        for (name, bauen) in galerie_zustaende() {
            html.push_str(&format!("<h2>{name}</h2><div class=z>"));
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let (mut app, tmp) = bauen(); app.einst.zoom = 1.0; let mut dateien = vec![];
                for (schema, sn) in [(Schema::Dunkel, "dunkel"), (Schema::Hell, "hell"), (Schema::Kontrast, "kontrast")] { app.einst.schema = schema; for (b, h) in groessen {
                    let f = format!("{name}-{sn}-{b}x{h}.png"); foto_gross(&mut app, &[], aus.join(&f).to_str().unwrap(), b, h); dateien.push((f, b, h, sn)) } }
                let _ = std::fs::remove_dir_all(&tmp); dateien
            }));
            match r { Ok(d) => for (f, b, h, sn) in d { n += 1; html.push_str(&format!("<figure><a href=\"{f}\"><img src=\"{f}\" loading=lazy></a><figcaption>{} · {b}×{h}</figcaption></figure>", sn)) }, Err(_) => { fehler.push(name); html.push_str("<p>Zustand ließ sich nicht aufbauen</p>") } }
            html.push_str("</div>");
        }
        std::fs::write(aus.join("index.html"), html).unwrap(); eprintln!("Galerie: {n} Bilder in {} (Fehler: {fehler:?})", aus.canonicalize().unwrap().display()); assert!(fehler.is_empty(), "Zustände ohne Bild: {fehler:?}");
    }
    /// Farbkanäle: Rot, Grün, Blau erscheinen im PNG als Rot, Grün, Blau (kein Vertauschen von R und B)
    #[test] fn bildfarben_rot_gruen_blau_sind_nicht_vertauscht() {
        let tmp = std::env::temp_dir().join(format!("vcgui-farben-{}.png", std::process::id()));
        let feld = |c: iced::Color| container(space::vertical()).width(40).height(40).style(move |_| container::Style { background: Some(c.into()), ..Default::default() });
        let el: Element<Msg> = row![feld(iced::Color::from_rgb8(255, 0, 0)), feld(iced::Color::from_rgb8(0, 255, 0)), feld(iced::Color::from_rgb8(0, 0, 255)), feld(iced::Color::from_rgb8(10, 20, 200))].spacing(0).into();
        foto_element(el, 220, 100, iced::Color::BLACK, tmp.to_str().unwrap());
        let pix = tiny_skia::Pixmap::load_png(&tmp).unwrap(); let at = |x: u32, y: u32| { let p = pix.pixel(x, y).unwrap(); (p.red(), p.green(), p.blue()) };
        assert_eq!(at(40, 40), (255, 0, 0), "Rot"); assert_eq!(at(80, 40), (0, 255, 0), "Grün"); assert_eq!(at(120, 40), (0, 0, 255), "Blau"); assert_eq!(at(160, 40), (10, 20, 200), "Mischfarbe");
        let _ = std::fs::remove_file(&tmp);
    }
    // ---------------------------------------------------------------------------------------------- Layout-Prüfung
    /// Knoten des Widget-Baums mit Fenstergrenzen, gesammelt über `Operation`
    #[derive(Clone, Debug)] struct Knoten { art: &'static str, grenzen: iced::Rectangle, text: String, in_scroll: bool, clip: Option<iced::Rectangle>, versatz: iced::Vector, tiefe: usize }
    struct Sammler { knoten: Vec<Knoten>, tiefe: usize, scroll: Vec<usize>, clips: Vec<iced::Rectangle> }
    impl iced::advanced::widget::Operation for Sammler {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) { self.tiefe += 1; operate(self); self.tiefe -= 1; if self.scroll.last().is_some_and(|t| *t > self.tiefe) { self.scroll.pop(); self.clips.pop(); } }
        fn container(&mut self, _id: Option<&iced::advanced::widget::Id>, b: iced::Rectangle) { let s = !self.scroll.is_empty(); self.knoten.push(Knoten { art: "container", grenzen: b, text: String::new(), in_scroll: s, clip: self.clips.last().copied(), versatz: iced::Vector::ZERO, tiefe: self.tiefe }) }
        fn scrollable(&mut self, _id: Option<&iced::advanced::widget::Id>, b: iced::Rectangle, _c: iced::Rectangle, _t: iced::Vector, _s: &mut dyn iced::advanced::widget::operation::Scrollable) { self.scroll.push(self.tiefe + 1); self.clips.push(b); self.knoten.push(Knoten { art: "scroll", grenzen: b, text: String::new(), in_scroll: false, clip: None, versatz: _t, tiefe: self.tiefe }) }
        fn focusable(&mut self, _id: Option<&iced::advanced::widget::Id>, b: iced::Rectangle, _s: &mut dyn iced::advanced::widget::operation::Focusable) { let s = !self.scroll.is_empty(); self.knoten.push(Knoten { art: "fokus", grenzen: b, text: String::new(), in_scroll: s, clip: self.clips.last().copied(), versatz: iced::Vector::ZERO, tiefe: self.tiefe }) }
        fn text_input(&mut self, _id: Option<&iced::advanced::widget::Id>, b: iced::Rectangle, _s: &mut dyn iced::advanced::widget::operation::TextInput) { let s = !self.scroll.is_empty(); self.knoten.push(Knoten { art: "eingabe", grenzen: b, text: String::new(), in_scroll: s, clip: self.clips.last().copied(), versatz: iced::Vector::ZERO, tiefe: self.tiefe }) }
        fn text(&mut self, _id: Option<&iced::advanced::widget::Id>, b: iced::Rectangle, t: &str) { let s = !self.scroll.is_empty(); self.knoten.push(Knoten { art: "text", grenzen: b, text: t.to_string(), in_scroll: s, clip: self.clips.last().copied(), versatz: iced::Vector::ZERO, tiefe: self.tiefe }) }
    }
    /// Wie `knoten_sammeln`, aber mit dem Zustand der Widgets aus `simuliere` (Scrollstellung, Fokus): misst, was echte Ereignisse bewirkt haben
    fn knoten_mit_zustand(app: &mut App) -> Vec<Knoten> {
        use iced_runtime::user_interface::UserInterface;
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new({ schriften_laden(); vc_gui_ui::fonts::UI }, iced::Pixels(16.0)));
        let cache = CACHE.with(|c| c.borrow_mut().take()).unwrap_or_default(); let grenze = iced::Size::new(app.fenster.width / app.einst.zoom, app.fenster.height / app.einst.zoom);
        let view = app.view(); let mut ui = UserInterface::build(view, grenze, cache, &mut renderer);
        let mut s = Sammler { knoten: vec![], tiefe: 0, scroll: vec![], clips: vec![] }; ui.operate(&renderer, &mut s); let k = s.knoten; CACHE.with(|c| *c.borrow_mut() = Some(ui.into_cache())); k
    }
    fn knoten_sammeln(app: &mut App, b: u32, h: u32) -> Vec<Knoten> {
        use iced_runtime::user_interface::{Cache, UserInterface};
        app.fenster = iced::Size::new(b as f32 / app.einst.zoom, h as f32 / app.einst.zoom);
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new({ schriften_laden(); vc_gui_ui::fonts::UI }, iced::Pixels(16.0)));
        let view = app.view(); let mut ui = UserInterface::build(view, iced::Size::new(b as f32, h as f32), Cache::default(), &mut renderer);
        let mut s = Sammler { knoten: vec![], tiefe: 0, scroll: vec![], clips: vec![] }; ui.operate(&renderer, &mut s); s.knoten
    }
    /// Befunde der Layout-Prüfung für eine Ansicht
    fn layout_befunde(knoten: &[Knoten], b: f32, h: f32, ueberlagerung: bool) -> Vec<String> {
        let mut out = vec![]; let kurz = |t: &str| -> String { t.chars().take(28).collect() };
        for (i, k) in knoten.iter().enumerate() {
            let g = k.grenzen;
            if k.art == "text" && !k.text.trim().is_empty() {
                if g.width < 1.0 || g.height < 1.0 { out.push(format!("unsichtbar (Breite 0): „{}“", kurz(&k.text))); continue }
                // kurzer Text, der umbricht (Knopfbeschriftung, Chip, Reiter …)
                if k.text.chars().count() <= 30 && !k.text.contains('\n') && g.height > 26.0 { out.push(format!("kurzer Text bricht um: „{}“", kurz(&k.text))) }
                // Text über den Rand seines Behälters hinaus
                if let Some(p) = knoten[..i].iter().rev().find(|c| c.art == "container" && c.tiefe + 1 == k.tiefe).filter(|_| !k.in_scroll) { if g.x + g.width > p.grenzen.x + p.grenzen.width + 1.5 || g.x < p.grenzen.x - 1.5 { out.push(format!("Text ragt aus dem Behälter: „{}“", kurz(&k.text))) } }
            }
            if !k.in_scroll && k.art != "scroll" && (g.x + g.width > b + 1.0 || g.y + g.height > h + 1.0 || g.x < -1.0 || g.y < -1.0) && g.width > 0.0 { out.push(format!("außerhalb des Fensters: {}", if k.text.is_empty() { k.art.to_string() } else { format!("„{}“", kurz(&k.text)) })) }
            if (k.art == "eingabe" || k.art == "fokus") && g.width > 0.0 && (g.height < 20.0 || g.width < 20.0) { out.push(format!("Klickfläche zu klein: {}", k.art)) }
        }
        if !ueberlagerung {
            let texte: Vec<&Knoten> = knoten.iter().filter(|k| k.art == "text" && !k.text.trim().is_empty() && k.grenzen.width >= 1.0).collect();
            for (n, a) in texte.iter().enumerate() { for c in &texte[n + 1..] { if a.in_scroll != c.in_scroll { continue } let sichtbar = |k: &Knoten| -> Option<iced::Rectangle> { match k.clip { None => Some(k.grenzen), Some(cl) => cl.intersection(&k.grenzen).filter(|i| i.width > 1.0 && i.height > 1.0) } }; let (Some(ga), Some(gc)) = (sichtbar(a), sichtbar(c)) else { continue }; let (x0, x1, y0, y1) = (ga.x.max(gc.x), (ga.x + ga.width).min(gc.x + gc.width), ga.y.max(gc.y), (ga.y + ga.height).min(gc.y + gc.height));
                if x1 - x0 > 3.0 && y1 - y0 > 3.0 { { if std::env::var_os("VC_DBG_LAYOUT").is_some() { eprintln!("{:?} {:?} {:?} {:?}", a.grenzen, a.clip, c.grenzen, c.clip) } out.push(format!("Texte überlappen: „{}“ und „{}“", kurz(&a.text), kurz(&c.text))) } } } }
        }
        // Kopfzeilenknöpfe sitzen je nach Textbreite des Projektnamens knapp an der Grenze: als ein Befund führen
        for f in out.iter_mut() { if ["„▶ Starten“", "„● Debuggen“", "„▣ Bauen“", "„⚙“", "Befehl oder Datei suchen"].iter().any(|n| f.contains(n)) && (f.starts_with("unsichtbar") || f.starts_with("kurzer Text")) { *f = "Kopfzeilenknöpfe gequetscht (Starten, Debuggen, Bauen, Suchfeld)".into() } }
        out.sort(); out.dedup(); out
    }
    #[test] fn einstellungen_seiten_entwurf_speichern_verwerfen_ohne_echte_datei() {
        use setui::{Seite, SetAkt, SEITEN};
        let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Einstellungen); assert_eq!(app.tabs.get(app.aktiv), Some(&Tab::Einstellungen));
        let datei = app.satz.datei.clone().expect("Testdatei"); assert!(datei.starts_with(std::env::temp_dir()), "im Test nie die echte Einstellungsdatei: {datei:?}");
        for (s, _) in SEITEN { let _ = app.update(Msg::Set(SetAkt::Seite(s))); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().filter(|k| k.art == "text").count() > 3, "{s:?} bleibt leer"); }
        // Entwurf: ungespeichert, Verwerfen stellt her, Speichern schreibt nur die Testdatei
        let _ = app.update(Msg::Set(SetAkt::Seite(Seite::Toolchain))); let _ = app.update(Msg::Set(SetAkt::AsmArgs("-D X=1".into()))); assert!(app.satz.geaendert());
        let _ = app.update(Msg::Set(SetAkt::Verwerfen)); assert!(!app.satz.geaendert() && app.satz.entwurf.assembler.args.is_empty());
        let _ = app.update(Msg::Set(SetAkt::AsmArgs("-D X=1".into()))); let _ = app.update(Msg::Set(SetAkt::Speichern)); assert!(!app.satz.geaendert());
        let t = std::fs::read_to_string(&datei).unwrap(); assert!(t.contains("-D X=1"), "{t}");
        // Sofortwirkung speichert einzeln und nimmt keinen anderen Entwurf mit
        let _ = app.update(Msg::Set(SetAkt::AsmArgs("-D Y=2".into()))); let _ = app.update(Msg::Set(SetAkt::DebugLog(true)));
        let t = std::fs::read_to_string(&datei).unwrap(); assert!(t.contains("debug_log") && t.contains("-D X=1") && !t.contains("Y=2"), "{t}");
        assert!(app.satz.geaendert(), "der andere Entwurf bleibt ungespeichert");
        let _ = app.update(Msg::Set(SetAkt::HilfeBilder("never".into()))); assert_eq!(app.satz.gespeichert.help.images, "never"); let _ = app.update(Msg::Set(SetAkt::HilfeBilder("ask".into()))); assert!(app.satz.gespeichert.help.images.is_empty());
        // Kürzel: Suche filtert die Tabelle
        let _ = app.update(Msg::Set(SetAkt::Seite(Seite::Kuerzel))); let alle = knoten_sammeln(&mut app, 1280, 800).len(); let _ = app.update(Msg::Set(SetAkt::KuerzelSuche("speichern".into()))); assert!(knoten_sammeln(&mut app, 1280, 800).len() < alle);
        // „Im Ordner zeigen“ läuft über aussen (aufgezeichnet)
        let _ = aussen::aufgezeichnet(); let _ = app.update(Msg::Set(SetAkt::OrdnerZeigen)); assert!(aussen::aufgezeichnet().iter().any(|a| matches!(a, aussen::Aufruf::ImOrdnerZeigen(_))) || vc_core::userdirs::config_dir().is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn rom_seite_status_vorschau_kopieren_und_festnageln_im_tempordner() {
        use romui::{RomAkt, StandPtr}; use setui::{Seite, SetAkt};
        let (mut app, tmp) = studio_app(); let roms = tmp.join("roms-test"); std::fs::create_dir_all(roms.join("mega65")).unwrap(); std::fs::write(roms.join("mega65/mega65.rom"), vec![7u8; 131072]).unwrap();
        app.satz.roms.test_ordner = Some(roms.clone()); let _ = app.befehl(Cmd::Einstellungen); let _ = app.update(Msg::Set(SetAkt::Seite(Seite::Roms))); assert!(app.satz.roms.laedt, "erstes Öffnen liest den Stand im Hintergrund");
        let laden = |app: &mut App| { let st = vc_roms::RomStore::new(app.rom_env()); let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Geladen(StandPtr(std::sync::Arc::new(st)))))); };
        laden(&mut app); assert!(app.satz.roms.stand.is_some() && !app.satz.roms.laedt);
        for i in 0..vc_roms::systems::SYSTEMS.len() { let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::System(i)))); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.art == "text" && k.text.contains("ROM-Ordner")), "System {i}"); }
        let m65 = vc_roms::systems::SYSTEMS.iter().position(|s| s.id == "mega65").unwrap(); let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::System(m65))));
        // abgelegte Datei: Vorschau, vor der Bestätigung wird nichts kopiert
        let f = tmp.join("neu.rom"); std::fs::write(&f, vec![9u8; 131072]).unwrap();
        let _ = app.update(Msg::Abgelegt(f.clone())); assert!(app.satz.roms.vorschau.is_some(), "Ablegen auf der ROM-Seite zeigt eine Vorschau, kein Projektimport"); assert!(app.import.is_none());
        let kopierbar = app.satz.roms.vorschau.as_ref().unwrap().items.iter().filter(|(i, _)| i.action == vc_roms::import::Action::Copy).count();
        assert!(!roms.join("mega65/neu.rom").exists());
        let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Alle(false)))); assert!(app.satz.roms.vorschau.as_ref().unwrap().items.iter().all(|(_, h)| !*h)); let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Alle(true))));
        let _ = knoten_sammeln(&mut app, 1280, 800);
        if kopierbar > 0 { let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Kopieren))); assert!(app.satz.roms.meldung.as_ref().is_some_and(|m| m.0), "{:?}", app.satz.roms.meldung); assert!(std::fs::read_dir(roms.join("mega65")).unwrap().count() >= 2, "Datei liegt im Test-ROM-Ordner"); }
        // „mitbenutzen“ speichert einzeln, nur in der Testdatei
        let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Fremde(true)))); assert!(app.satz.gespeichert.roms.use_foreign); assert!(std::fs::read_to_string(app.satz.datei.as_ref().unwrap()).unwrap().contains("use_foreign"));
        let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Fremde(false))));
        // Suche läuft nur auf Wunsch und liefert eine Vorschau
        let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Suchen))); assert!(app.satz.roms.sucht.is_some()); let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::Gefunden(vec![], true)))); assert!(app.satz.roms.sucht.is_none() && app.satz.roms.vorschau.as_ref().is_some_and(|v| v.items.is_empty()));
        let _ = app.update(Msg::Set(SetAkt::Rom(RomAkt::VorschauZu))); assert!(app.satz.roms.vorschau.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    fn ziele_app() -> (App, std::path::PathBuf) {
        use zielui::ZielAkt; use setui::{Seite, SetAkt};
        let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Einstellungen); let _ = app.update(Msg::Set(SetAkt::Seite(Seite::Ziele)));
        for id in ["c64", "mega65", "atari8"] { let _ = app.update(Msg::Set(SetAkt::Ziel(ZielAkt::Hinzu(id.into())))); }
        let _ = app.update(Msg::Set(SetAkt::Ziel(ZielAkt::Wahl("mega65".into())))); (app, tmp)
    }
    #[test] fn ziele_seite_systeme_emulatoren_hardware_und_proben_nur_ueber_aussen() {
        use zielui::{Feld, Probe, ZielAkt}; use setui::SetAkt;
        let (mut app, tmp) = ziele_app(); let akt = |app: &mut App, a: ZielAkt| { let _ = app.update(Msg::Set(SetAkt::Ziel(a))); };
        assert!(app.satz.entwurf.target("c64").is_some() && app.satz.entwurf.target("mega65").is_some(), "Systeme hinzugefügt");
        // Emulator: hinzufügen, benennen, Standard, Test-Start nur als Probe (Aufruf läuft über aussen und wird im Test nur aufgezeichnet)
        akt(&mut app, ZielAkt::Wahl("c64".into())); let art = targets_first_kind("c64"); akt(&mut app, ZielAkt::EmuPlus(art.clone()));
        let eid = app.satz.entwurf.target("c64").unwrap().emulators.last().unwrap().id.clone(); akt(&mut app, ZielAkt::EmuName(eid.clone(), "Mein VICE".into())); akt(&mut app, ZielAkt::EmuArgs(eid.clone(), "-warp".into())); akt(&mut app, ZielAkt::EmuStandard(eid.clone()));
        let e = app.satz.entwurf.target("c64").unwrap().emulators.iter().find(|e| e.id == eid).unwrap().clone(); assert!(e.standard && e.name == "Mein VICE" && e.args == "-warp");
        let _ = aussen::aufgezeichnet(); assert!(aussen::emulator_test(&e).is_ok()); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::EmulatorTest(eid.clone())), "Test-Start wird nur aufgezeichnet");
        akt(&mut app, ZielAkt::ProbeErg(zielui::emu_key(&eid), Ok("läuft".into()))); assert_eq!(app.satz.ziel.proben.get(&zielui::emu_key(&eid)), Some(&Probe::Ok("läuft".into())));
        // Hardware (MEGA65): hinzufügen, Felder, Port-Liste (Attrappe), Verbindungstest nur über aussen
        akt(&mut app, ZielAkt::Wahl("mega65".into())); akt(&mut app, ZielAkt::MaschPlus("mega65".into())); let mid = app.satz.entwurf.machines.last().unwrap().id.clone();
        akt(&mut app, ZielAkt::MaschFeld(mid.clone(), Feld::Name, "Mein MEGA65".into())); akt(&mut app, ZielAkt::PortsErg(aussen::ports_suchen())); akt(&mut app, ZielAkt::PortWahl(mid.clone(), "/dev/cu.test-port".into())); akt(&mut app, ZielAkt::MaschVerb(mid.clone(), "eth".into())); akt(&mut app, ZielAkt::MaschVerbFuer(mid.clone(), "debug", "usb".into()));
        let m = app.satz.entwurf.machine(&mid).unwrap().clone(); assert!(m.name == "Mein MEGA65" && m.device == "/dev/cu.test-port" && m.link == "eth" && m.link_debug == "usb");
        let _ = aussen::aufgezeichnet(); assert!(aussen::hardware_test(&m, None).is_ok() && aussen::aufgezeichnet().contains(&aussen::Aufruf::HardwareTest(mid.clone())));
        akt(&mut app, ZielAkt::StartZiel(mid.clone())); assert_eq!(app.satz.entwurf.target("mega65").unwrap().start_on, mid);
        // Download nur nach Bestätigung: erst Frage, dann Installieren (Aufruf nur aufgezeichnet)
        akt(&mut app, ZielAkt::InstallFrage(Some("vice@3.9".into()))); assert_eq!(app.satz.ziel.install_frage.as_deref(), Some("vice@3.9")); akt(&mut app, ZielAkt::InstallFrage(None)); assert!(app.satz.ziel.install_frage.is_none());
        let _ = aussen::aufgezeichnet(); assert!(aussen::emulator_installieren("vice@3.9").is_ok() && aussen::aufgezeichnet().contains(&aussen::Aufruf::EmulatorInstall("vice@3.9".into())));
        // alle Systeme zeichnen
        for id in ["c64", "mega65", "atari8"] { akt(&mut app, ZielAkt::Wahl(id.into())); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().filter(|k| k.art == "text").count() > 10, "{id}"); }
        // Filter, Deaktivieren, Speichern in die Testdatei
        akt(&mut app, ZielAkt::Filter("mega".into())); let _ = knoten_sammeln(&mut app, 1280, 800); akt(&mut app, ZielAkt::Filter(String::new()));
        akt(&mut app, ZielAkt::Wahl("atari8".into())); akt(&mut app, ZielAkt::Weg); assert!(app.satz.entwurf.target("atari8").is_none() && app.satz.ziel.system != "atari8");
        assert!(app.satz.geaendert()); let _ = app.update(Msg::Set(SetAkt::Speichern)); let t = std::fs::read_to_string(app.satz.datei.as_ref().unwrap()).unwrap(); assert!(t.contains("Mein VICE") && t.contains("Mein MEGA65"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn startparameter_katalog_auswahl_frei_platzhalter_und_vorschau_im_entwurf() {
        use argsui::ArgAkt; use zielui::ZielAkt; use setui::{Seite, SetAkt};
        let (mut app, tmp) = ziele_app(); let akt = |app: &mut App, a: ZielAkt| { let _ = app.update(Msg::Set(SetAkt::Ziel(a))); }; let arg = |app: &mut App, a: ArgAkt| { let _ = app.update(Msg::Set(SetAkt::Arg(a))); };
        akt(&mut app, ZielAkt::Wahl("c64".into())); akt(&mut app, ZielAkt::EmuPlus(targets_first_kind("c64"))); let eid = app.satz.entwurf.target("c64").unwrap().emulators.last().unwrap().id.clone();
        let kat = vc_ide::args::catalog_of_kind(&app.satz.entwurf.target("c64").unwrap().emulators.last().unwrap().kind).expect("Katalog für VICE"); let p = kat.params.iter().find(|p| p.ty == vc_ide::args::ParamType::Switch && p.off_flag.is_none()).expect("Schalter");
        // aufklappen und Werte setzen
        let k0 = knoten_sammeln(&mut app, 1280, 800); assert!(!k0.iter().any(|k| k.art == "text" && k.text.starts_with("Katalog ")), "ohne gesetzte Werte zugeklappt");
        arg(&mut app, ArgAkt::Auf(format!("args:{eid}"), false)); let k1 = knoten_sammeln(&mut app, 1280, 800); assert!(k1.iter().any(|k| k.art == "text" && k.text.starts_with("Katalog ")), "aufgeklappt"); arg(&mut app, ArgAkt::Auf(format!("args:{eid}"), true));
        arg(&mut app, ArgAkt::Wert(eid.clone(), p.id.clone(), "on".into())); assert_eq!(app.satz.entwurf.target("c64").unwrap().emulators.last().unwrap().opts.get(&p.id).map(String::as_str), Some("on"));
        arg(&mut app, ArgAkt::Frei(eid.clone(), "-foo".into())); arg(&mut app, ArgAkt::Platzhalter(eid.clone(), "project".into()));
        let e = app.satz.entwurf.target("c64").unwrap().emulators.last().unwrap().clone(); assert!(e.args.contains("-foo") && e.args.contains("{project}"), "{:?}", e.args);
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.art == "text" && k.text.starts_with("Katalog ")), "Katalog-Kopf");
        arg(&mut app, ArgAkt::Zurueck(eid.clone())); let e = app.satz.entwurf.target("c64").unwrap().emulators.last().unwrap().clone(); assert!(e.opts.is_empty() && e.args.is_empty());
        // Assembler-Optionen auf der Toolchain-Seite
        let _ = app.update(Msg::Set(SetAkt::Seite(Seite::Toolchain))); arg(&mut app, ArgAkt::Frei("asm".into(), "-D DEBUG=1".into())); assert_eq!(app.satz.entwurf.assembler.args, "-D DEBUG=1"); let _ = knoten_sammeln(&mut app, 1280, 800);
        arg(&mut app, ArgAkt::Zurueck("asm".into())); assert!(app.satz.entwurf.assembler.args.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn parametereditor_bleibt_beim_tippen_aufgeklappt_auch_beim_leeren_des_feldes() {
        use argsui::ArgAkt; use setui::{Seite, SetAkt};
        let (mut app, tmp) = studio_app(); einstellungen::test_konfig(tmp.join("konf")); let _ = app.befehl(Cmd::Einstellungen); let _ = app.update(Msg::Set(SetAkt::Seite(Seite::Toolchain)));
        let kopf = |app: &mut App| -> Option<String> { knoten_sammeln(app, 1280, 800).into_iter().find(|k| k.art == "text" && (k.text.starts_with("▾ ") || k.text.starts_with("▸ ")) && k.text.contains("Assembler-Optionen")).map(|k| k.text) };
        let k0 = kopf(&mut app).expect("Kopf des Parametereditors"); assert!(k0.starts_with("▸ "), "zu Beginn zu: {k0}");
        let _ = app.update(Msg::Set(SetAkt::Arg(ArgAkt::Auf("args:asm".into(), false)))); assert!(kopf(&mut app).unwrap().starts_with("▾ "), "aufgeklappt");
        // Zeichen für Zeichen tippen (jedes Zeichen ist eine Änderung des Eingabefeldes), dann wieder löschen: bleibt aufgeklappt
        let ziel = "-D X=1"; for i in 1..=ziel.chars().count() { let t: String = ziel.chars().take(i).collect(); let _ = app.update(Msg::Set(SetAkt::Arg(ArgAkt::Frei("asm".into(), t.clone())))); assert!(kopf(&mut app).unwrap().starts_with("▾ "), "bleibt offen nach {t:?}"); assert_eq!(app.satz.entwurf.assembler.args, t); }
        for i in (0..ziel.chars().count()).rev() { let t: String = ziel.chars().take(i).collect(); let _ = app.update(Msg::Set(SetAkt::Arg(ArgAkt::Frei("asm".into(), t.clone())))); assert!(kopf(&mut app).unwrap().starts_with("▾ "), "bleibt offen auch beim Löschen bis {t:?}"); }
        // ausdrücklich zuklappen funktioniert weiter
        let _ = app.update(Msg::Set(SetAkt::Arg(ArgAkt::Auf("args:asm".into(), true)))); assert!(kopf(&mut app).unwrap().starts_with("▸ "), "zugeklappt");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Start ohne Projektangabe (wie das ausgelieferte Programm): Startseite, kein Reiter, keine Meldung, nichts in „Zuletzt geöffnet“; kein Pfad aus dem Quellbaum
    #[test] fn start_ohne_argument_zeigt_die_startseite_ohne_projekt() {
        let tmpf = std::env::temp_dir().join(format!("vcgui-start-ohne-{}.json", std::process::id())); let _ = std::fs::remove_file(&tmpf); vc_ide_model::recent::umleiten(Some(tmpf.clone()));
        let (mut app, _) = App::neu_aus(vec!["vintagecoder".into()], false);
        assert!(app.tabs.is_empty() && app.editoren.is_empty(), "kein Reiter"); assert!(app.root.ends_with("vintagecoder-ohne-projekt") && !app.root.to_string_lossy().contains("examples"), "{:?}", app.root);
        assert!(app.proj.pf.is_none() && app.log.iter().all(|l| !l.to_lowercase().contains("fehler")), "{:?}", app.log);
        assert!(vc_ide_model::recent::load(Some(&app.data)).is_empty() && !tmpf.exists(), "nichts in Zuletzt geöffnet geschrieben");
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Neues Projekt") && k.iter().any(|k| k.text == "Zuletzt geöffnete Projekte"), "Startseite");
        assert!(k.iter().any(|k| k.text == "Kein Projekt") && !k.iter().any(|k| k.text.contains("ohne-projekt")), "Kopf nennt kein Projekt statt des Platzhalterordners");
        // der Platzhaltername steht in keiner Breite irgendwo (Kopf, Statuszeile, Bereiche, Menüs), auch nicht mit geöffneter Befehlspalette
        for b in [480.0, 800.0, 1280.0, 1920.0] { let k = knoten_sammeln(&mut app, b as u32, 800); assert!(!k.iter().any(|k| k.text.contains("ohne-projekt") || k.text.contains("vcgui")), "Breite {b}"); }
        let _ = app.befehl(Cmd::Palette); let k = knoten_sammeln(&mut app, 1280, 800); assert!(!k.iter().any(|k| k.text.contains("ohne-projekt")), "Befehlspalette");
        // Entwicklungsmodus (cargo run, Tests): öffnet wie früher das Beispiel
        let (dev, _) = App::neu_aus(vec!["vintagecoder".into()], true); assert!(dev.root.ends_with("examples/c64/prallball") && !dev.tabs.is_empty());
        let _ = std::fs::remove_file(&tmpf);
    }
    fn targets_first_kind(sys: &str) -> String { vc_ide::targets::system(sys).unwrap().kinds().iter().find(|k| vc_ide::emulators::enabled(k.id)).map(|k| k.id.to_string()).expect("Emulator-Sorte") }
    #[test] fn startseite_zuletzt_geoeffnet_anheften_entfernen_beispiele_und_hardware_test_ueber_aussen() {
        use startui::{Probe, StartAkt};
        let (mut app, tmp) = studio_app(); let tmpf = tmp.join("recent-test.json"); vc_ide_model::recent::umleiten(Some(tmpf.clone()));
        // Leerzustand = Startseite
        app.tabs.clear(); app.aktiv = 0; let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.art == "text" && k.text == "Neues Projekt") && k.iter().any(|k| k.text == "Zuletzt geöffnete Projekte"), "Startseite im Leerzustand");
        // Reiter „Start“ per Befehl
        let _ = app.befehl(Cmd::Startseite); assert_eq!(app.tabs.get(app.aktiv), Some(&Tab::Start));
        // zuletzt geöffnet: zwei Projekte eintragen
        let (a, b) = (tmp.join("pa"), tmp.join("pb")); std::fs::create_dir_all(&a).unwrap(); std::fs::create_dir_all(&b).unwrap();
        vc_ide_model::recent::touch(&mut app.start.recent, &a, "Projekt A", Some(&app.data)); vc_ide_model::recent::touch(&mut app.start.recent, &b, "Projekt B", Some(&app.data));
        { let mut pm = tiny_skia::Pixmap::new(16, 12).unwrap(); pm.fill(tiny_skia::Color::from_rgba8(200, 60, 60, 255)); pm.save_png(a.join("projektbild.png")).unwrap(); }
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Projekt A") && k.iter().any(|k| k.text == "Projekt B"), "mit Projektbild gezeichnet");
        let _ = app.update(Msg::StartSeite(StartAkt::Pin(1))); assert!(app.start.recent.iter().filter(|r| r.pinned).count() == 1, "genau eins angeheftet"); assert!(std::fs::read_to_string(&tmpf).unwrap().contains("\"pinned\": true"), "Testdatei, nicht die echte");
        let n = app.start.recent.len(); let _ = app.update(Msg::StartSeite(StartAkt::Weg(0))); assert_eq!(app.start.recent.len(), n - 1);
        // Neues Projekt: noch kein Assistent (4b), kein Absturz
        let _ = app.update(Msg::StartSeite(StartAkt::Neu)); assert!(app.neu_projekt.is_some(), "Neues Projekt öffnet den Dialog"); let _ = app.update(Msg::NeuProj(neuprojekt::NpAkt::Abbruch)); assert!(app.neu_projekt.is_none());
        // Beispiele aufklappen
        let _ = app.update(Msg::StartSeite(StartAkt::Beispiele)); let _ = knoten_sammeln(&mut app, 1280, 800);
        // Hardware: Test nur über aussen (im Test aufgezeichnet)
        let m = vc_ide::settings::Machine { id: "m1".into(), kind: "ultimate64".into(), name: "U64".into(), host: "192.0.2.1".into(), ..Default::default() }; app.satz.gespeichert.machines.push(m);
        let _ = aussen::aufgezeichnet(); let _ = app.update(Msg::StartSeite(StartAkt::Testen("m1".into()))); assert_eq!(app.start.proben.get("m1"), Some(&Probe::Laeuft));
        let _ = app.update(Msg::StartSeite(StartAkt::ProbeErg("m1".into(), Ok("ok".into())))); assert_eq!(app.start.proben.get("m1"), Some(&Probe::Ok)); let _ = knoten_sammeln(&mut app, 1280, 800);
        // Schnellreferenz öffnet das Hilfe-Panel
        let _ = app.update(Msg::StartSeite(StartAkt::Hilfe("start"))); assert_eq!(app.panel_seite, PanelSeite::Hilfe);
        // schmal: einspaltig, nichts fehlt
        let k = knoten_sammeln(&mut app, 720, 800); assert!(k.iter().any(|k| k.text == "Schnellreferenz"));
        vc_ide_model::recent::umleiten(None); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn kodierung_und_zeilenende_bleiben_beim_speichern_erhalten_und_lassen_sich_umschalten() {
        let (mut app, tmp) = studio_app();
        std::fs::write(tmp.join("crlf.asm"), b"start:\r\n  nop\r\n").unwrap(); std::fs::write(tmp.join("latin.asm"), [b';', b' ', 0xE4, b'\n', b'n', b'o', b'p', b'\n']).unwrap(); app.projekt_neu_laden();
        app.oeffne("crlf.asm"); assert!(app.textmeta["crlf.asm"].crlf && !app.textmeta["crlf.asm"].latin1); app.ed().unwrap().insert("x"); app.speichern(); assert_eq!(std::fs::read(tmp.join("crlf.asm")).unwrap(), b"xstart:\r\n  nop\r\n", "CRLF bleibt");
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.art == "text" && k.text == "CRLF") && k.iter().any(|k| k.text == "UTF-8"), "Statusleiste zeigt Zeilenende und Kodierung");
        let _ = app.befehl(Cmd::Zeilenende); assert!(!app.textmeta["crlf.asm"].crlf); app.speichern(); assert_eq!(std::fs::read(tmp.join("crlf.asm")).unwrap(), b"xstart:\n  nop\n", "umgestellt auf LF");
        app.oeffne("latin.asm"); assert!(app.textmeta["latin.asm"].latin1); app.ed().unwrap().insert("y"); app.speichern(); assert_eq!(std::fs::read(tmp.join("latin.asm")).unwrap(), [b'y', b';', b' ', 0xE4, b'\n', b'n', b'o', b'p', b'\n'], "Latin-1 bleibt, nicht zu UTF-8 verbogen");
        let _ = app.befehl(Cmd::Kodierung); app.speichern(); assert_eq!(std::fs::read(tmp.join("latin.asm")).unwrap()[3..5], [0xC3, 0xA4], "auf UTF-8 umgestellt: ä als zwei Bytes");
        // nicht lesbare Datei (Ordner statt Datei) wird nie mit Fehlertext überschrieben
        std::fs::create_dir_all(tmp.join("ordner.asm")).unwrap(); app.projekt_neu_laden(); app.oeffne("ordner.asm"); assert!(app.lesefehler.contains("ordner.asm")); app.ed().unwrap().insert("zz"); app.speichern(); assert!(tmp.join("ordner.asm").is_dir(), "nichts überschrieben"); assert!(app.log.iter().any(|l| l.contains("nicht gespeichert")));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn unbenannte_reiter_notizen_speichern_unter_schliessen_sicherung_und_wiederherstellung() {
        let (mut app, tmp) = studio_app(); let sicherung = tmp.join("scratch-test.json"); app.scratch_datei = Some(sicherung.clone());
        let vorher: Vec<_> = std::fs::read_dir(&tmp).unwrap().flatten().map(|e| e.file_name()).collect();
        // neu: nur im Speicher, keine Datei im Projekt
        let _ = app.befehl(Cmd::NeuerReiter); let _ = app.befehl(Cmd::NeueNotiz); assert_eq!(app.scratch.len(), 2); assert_eq!(app.name().as_deref(), Some("Notiz 2"));
        assert_eq!(std::fs::read_dir(&tmp).unwrap().flatten().map(|e| e.file_name()).collect::<Vec<_>>().len(), vorher.len(), "keine Datei im Projektordner angelegt");
        // Inhalt tippen → Zwischensicherung (nur Reiter mit Inhalt)
        app.ed().unwrap().insert("# Meine Notiz"); app.scratch_sichern(); let t = std::fs::read_to_string(&sicherung).unwrap(); assert!(t.contains("Meine Notiz") && !t.contains("Unbenannt 1"), "{t}");
        // leeren Reiter schließen: ohne Rückfrage
        let i = app.tabs.iter().position(|t| matches!(t, Tab::Datei(n) if n == "Unbenannt 1")).unwrap(); let _ = app.update(Msg::Schliesse(i)); assert!(!app.editoren.contains_key("Unbenannt 1") && app.rueckfrage.is_none());
        // Reiter mit Inhalt schließen: Rückfrage; „Nein“ lässt ihn offen, „Ja“ verwirft und aktualisiert die Sicherung
        let i = app.tabs.iter().position(|t| matches!(t, Tab::Datei(n) if n == "Notiz 2")).unwrap(); let _ = app.update(Msg::Schliesse(i)); assert_eq!(app.rueckfrage, Some(Rueckfrage::ScratchWeg("Notiz 2".into()))); let _ = app.update(Msg::RueckfrageNein); assert!(app.editoren.contains_key("Notiz 2"));
        // Cmd+S = Speichern unter (Dialog), ungültiger Name bleibt im Dialog, gültiger legt die Datei an und benennt den Reiter um
        let _ = app.befehl(Cmd::Speichern); assert!(matches!(app.baum.as_ref().map(|b| &b.art), Some(BaumArt::SpeichernUnter(n)) if n == "Notiz 2")); assert_eq!(app.baum.as_ref().unwrap().text, "notiz.md");
        app.baum.as_mut().unwrap().text = "../aussen.md".into(); let _ = app.update(Msg::BaumOk); assert!(app.baum.as_ref().is_some_and(|b| !b.fehler.is_empty()), "Fehler bleibt im Dialog"); assert!(!tmp.parent().unwrap().join("aussen.md").exists());
        app.baum.as_mut().unwrap().text = "meine-notiz".into(); let _ = app.update(Msg::BaumOk); assert!(app.baum.is_none()); assert_eq!(std::fs::read_to_string(tmp.join("meine-notiz.md")).unwrap(), "# Meine Notiz"); assert_eq!(app.name().as_deref(), Some("meine-notiz.md")); assert!(app.scratch.is_empty() && !app.editoren.get("meine-notiz.md").unwrap().dirty);
        assert!(!std::fs::read_to_string(&sicherung).map(|t| t.contains("Meine Notiz")).unwrap_or(false), "nach dem Speichern keine Zwischenkopie mehr");
        // Wiederherstellung nach „Absturz“: Sicherung vorhanden → Dialog; „Wiederherstellen“ legt Reiter an, „Verwerfen“ löscht die Datei
        let e = vc_ide_model::scratchsave::Entry { name: "Unbenannt 7".into(), text: "lda #1".into(), note: false, saved_at: vc_ide_model::scratchsave::now_secs() }; vc_ide_model::scratchsave::save_to(&sicherung, &[e.clone()]);
        app.scratch_rest = Some(vc_ide_model::scratchsave::load_from(&sicherung, vc_ide_model::scratchsave::now_secs())); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Letzte Sitzung wiederherstellen?"));
        let _ = app.update(Msg::ScratchRest(1)); assert!(app.scratch.iter().any(|(_, n)| !*n) && app.editoren.values().any(|e| e.text() == "lda #1"));
        app.scratch_rest = Some(vec![e.clone()]); let _ = app.update(Msg::ScratchRest(0)); assert_eq!(app.scratch_kept.len(), 1, "„Nicht jetzt“ behält die Sicherung");
        app.scratch_rest = Some(vec![e]); let _ = app.update(Msg::ScratchRest(2)); assert!(app.scratch_kept.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn beenden_und_projektwechsel_fragen_bei_ungespeicherten_aenderungen() {
        let (mut app, tmp) = studio_app(); let _ = aussen::aufgezeichnet();
        // nichts geändert: Beenden ohne Rückfrage (Attrappe zeichnet nur auf)
        let _ = app.befehl(Cmd::Beenden); assert!(app.beenden_frage.is_none()); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::Beenden), "Beenden ohne Änderungen");
        // Datei ändern: Rückfrage, Abbrechen lässt alles; Fensterschließen und Cmd+Q gehen denselben Weg
        std::fs::write(tmp.join("a.asm"), "nop\n").unwrap(); app.projekt_neu_laden(); app.oeffne("a.asm"); app.ed().unwrap().insert("x");
        let _ = app.update(Msg::FensterZu); assert_eq!(app.beenden_frage.as_ref().map(|(l, f)| (l.clone(), f.clone())), Some((vec!["a.asm".to_string()], Folge::Beenden))); assert!(aussen::aufgezeichnet().is_empty(), "nicht beendet");
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "VintageCoder beenden?") && k.iter().any(|k| k.text == "Verwerfen"));
        let _ = app.update(Msg::BeendenAntwort(0)); assert!(app.beenden_frage.is_none() && app.editoren["a.asm"].dirty && aussen::aufgezeichnet().is_empty(), "Abbrechen: nichts passiert");
        let _ = app.update(Msg::Menue("beenden".into())); assert!(app.beenden_frage.is_some(), "Menü Beenden fragt auch");
        // Speichern: Datei geschrieben, dann beendet
        let _ = app.update(Msg::BeendenAntwort(1)); assert_eq!(std::fs::read_to_string(tmp.join("a.asm")).unwrap(), "xnop\n"); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::Beenden));
        // Verwerfen: Datei bleibt unverändert, dann beendet
        app.ed().unwrap().insert("y"); let _ = aussen::aufgezeichnet(); let _ = app.befehl(Cmd::Beenden); let _ = app.update(Msg::BeendenAntwort(2)); assert_eq!(std::fs::read_to_string(tmp.join("a.asm")).unwrap(), "xnop\n", "verworfen, nicht geschrieben"); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::Beenden));
        // Studio und unbenannter Reiter zählen mit
        let _ = app.befehl(Cmd::NeuerReiter); app.ed().unwrap().insert("lda #1"); let l = app.ungespeichert(); assert!(l.iter().any(|n| n == "Unbenannt 1") && l.iter().any(|n| n == "a.asm"), "{l:?}");
        // Projektwechsel mit Änderungen: Rückfrage statt stillem Speichern; Verwerfen wechselt ohne zu schreiben, Notizen bleiben
        let (neu, tmp2) = { let t = std::env::temp_dir().join(format!("vcgui-wechsel-{}", std::process::id())); let _ = std::fs::remove_dir_all(&t); std::fs::create_dir_all(&t).unwrap(); std::fs::write(t.join("b.asm"), "nop\n").unwrap(); (t.clone(), t) };
        let _ = app.update(Msg::Geoeffnet(Some(neu.clone()))); assert!(matches!(app.beenden_frage, Some((_, Folge::Projekt(ref p))) if *p == neu), "Wechsel fragt"); assert_eq!(app.root, tmp, "noch im alten Projekt");
        let _ = app.update(Msg::BeendenAntwort(0)); assert_eq!(app.root, tmp);
        let _ = app.update(Msg::Geoeffnet(Some(neu.clone()))); let _ = app.update(Msg::BeendenAntwort(2)); assert_eq!(app.root, neu, "gewechselt"); assert_eq!(std::fs::read_to_string(tmp.join("a.asm")).unwrap(), "xnop\n", "alte Datei nicht still gespeichert");
        // Notiz überlebt den Wechsel
        let _ = app.befehl(Cmd::NeueNotiz); app.ed().unwrap().insert("merk"); let (neu2, _) = (tmp.clone(), 0); let _ = app.update(Msg::Geoeffnet(Some(neu2))); assert!(app.beenden_frage.is_some()); let _ = app.update(Msg::BeendenAntwort(1)); assert!(app.scratch.iter().any(|(n, _)| app.editoren.get(n).is_some_and(|e| e.text() == "merk")), "Notiz bleibt über den Projektwechsel");
        let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&tmp2);
    }
    #[test] fn neues_projekt_zeichensatzwahl_steuert_vorschau_und_dateien() {
        use neuprojekt::{NpAkt, Sprache};
        let (mut app, tmp) = studio_app(); let ort = tmp.join("ort-zs"); std::fs::create_dir_all(&ort).unwrap();
        let _ = app.befehl(Cmd::ProjektNeu); app.neu_projekt.as_mut().unwrap().ort = ort.clone();
        let akt = |app: &mut App, a: NpAkt| { let _ = app.update(Msg::NeuProj(a)); };
        akt(&mut app, NpAkt::System("c64".into())); akt(&mut app, NpAkt::Sprache(Sprache::Asm)); akt(&mut app, NpAkt::Name("zsprobe".into()));
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Zeichensatz des Projekts") && k.iter().any(|k| k.text == "Nur Standard (eigen)"), "Abschnitt sichtbar");
        akt(&mut app, NpAkt::Zeichensatz("own".into())); let v = app.neu_projekt.as_ref().unwrap().vorschau(); assert!(v.iter().any(|f| f == "graphics/vc-standard/vc-c64-upper.vcchar") && v.iter().any(|f| f.ends_with("HERKUNFT.txt")), "{v:?}");
        akt(&mut app, NpAkt::Zeichensatz("none".into())); let v = app.neu_projekt.as_ref().unwrap().vorschau(); assert!(!v.iter().any(|f| f.contains("vcchar")), "{v:?}");
        akt(&mut app, NpAkt::Zeichensatz("own".into())); akt(&mut app, NpAkt::Ok);
        let dir = ort.join("zsprobe"); assert!(dir.join("graphics/vc-standard/vc-c64-upper.vcchar").is_file() && dir.join("graphics/vc-standard/HERKUNFT.txt").is_file(), "Kopie und Herkunft angelegt");
    }
    #[test] fn neues_projekt_dialog_vorlagen_filtern_anlegen_und_oeffnen_nur_im_tempordner() {
        use neuprojekt::{NpAkt, Sprache};
        let (mut app, tmp) = studio_app(); let ort = tmp.join("ort"); std::fs::create_dir_all(&ort).unwrap(); let _ = aussen::aufgezeichnet();
        let _ = app.befehl(Cmd::ProjektNeu); assert!(app.neu_projekt.is_some()); app.neu_projekt.as_mut().unwrap().ort = ort.clone();
        let akt = |app: &mut App, a: NpAkt| { let _ = app.update(Msg::NeuProj(a)); };
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Neues Projekt erstellen") && k.iter().any(|k| k.text == "Projekt erstellen und öffnen"));
        // Vorlagen folgen Zielsystem und Sprache
        akt(&mut app, NpAkt::System("c64".into())); akt(&mut app, NpAkt::Sprache(Sprache::Asm)); let w = app.neu_projekt.as_ref().unwrap(); assert!(w.gefiltert().iter().any(|&i| w.karten[i].name.contains("Steckmodul")));
        akt(&mut app, NpAkt::System("mega65".into())); let w = app.neu_projekt.as_ref().unwrap(); assert!(!w.gefiltert().iter().any(|&i| w.karten[i].variante.as_deref() == Some("cartridge")), "kein Steckmodul für MEGA65");
        akt(&mut app, NpAkt::System("atari8".into())); akt(&mut app, NpAkt::Sprache(Sprache::Basic)); assert_eq!(app.neu_projekt.as_ref().unwrap().dialekte(), vec!["atari_basic".to_string(), "turbo_basic_xl".to_string()]);
        for sys in ["c64", "c128", "vic20", "plus4", "mega65", "atari8"] { akt(&mut app, NpAkt::System(sys.into())); let _ = knoten_sammeln(&mut app, 1280, 800); let _ = knoten_sammeln(&mut app, 720, 800); }
        // ungültiger Name sperrt, gültig legt an: VIC-20 NTSC 16K, Diskette, README, Git (Attrappe), Öffnen
        akt(&mut app, NpAkt::System("vic20".into())); akt(&mut app, NpAkt::Sprache(Sprache::Asm)); akt(&mut app, NpAkt::Name("a b".into())); akt(&mut app, NpAkt::Ok); assert!(app.neu_projekt.is_some() && !ort.join("a b").exists(), "ungültiger Name legt nichts an");
        akt(&mut app, NpAkt::Name("vc20test".into())); akt(&mut app, NpAkt::Norm("ntsc".into())); let sp = app.neu_projekt.as_ref().unwrap().vic20_speicher().into_iter().find(|(k, _)| k == "16k").map(|x| x.0).unwrap_or_default(); akt(&mut app, NpAkt::Speicher(sp)); akt(&mut app, NpAkt::Disk("d64".into())); akt(&mut app, NpAkt::Git(true)); akt(&mut app, NpAkt::Readme(true));
        let hg = app.neu_projekt.as_ref().unwrap().vorschau(); assert!(hg.iter().any(|f| f == "vc20test.vcprj") && hg.iter().any(|f| f == "README.md") && hg.iter().any(|f| f == ".gitignore"), "{hg:?}");
        akt(&mut app, NpAkt::Ok); assert!(app.neu_projekt.is_none(), "{:?}", app.log.last());
        let dir = ort.join("vc20test"); let pf = vc_ide::projfile::ProjectFile::load(&dir.join("vc20test.vcprj")).unwrap(); assert_eq!(pf.targets[0].variant, "ntsc"); assert_eq!(pf.disk.as_ref().unwrap().format, "d64");
        assert!(dir.join("README.md").is_file() && dir.join(".gitignore").is_file()); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::GitInit(dir.clone())), "git init nur aufgezeichnet");
        // Rückfrage wegen ungespeicherter Änderungen im alten Projekt kommt zuerst oder das Projekt wurde geöffnet
        assert!(app.beenden_frage.is_some() || app.root == dir, "Projekt wird geöffnet (oder fragt vorher)");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn projekt_kopieren_dialog_prueft_kopiert_im_hintergrund_und_oeffnet_nur_im_tempordner() {
        use kopierdlg::KopAkt;
        let (mut app, _tmp0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); assert!(app.proj.pf.is_some(), "Beispiel hat Projektdatei"); let ort = tmp.parent().unwrap().join(format!("vcgui-kopien-{}", std::process::id())); let _ = std::fs::remove_dir_all(&ort); std::fs::create_dir_all(&ort).unwrap();
        let _ = app.befehl(Cmd::ProjektKopieren); assert!(app.kopier.is_some()); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Projekt kopieren") && k.iter().any(|k| k.text == "Kopieren"));
        let d = app.kopier.as_mut().unwrap(); assert!(d.name.ends_with("Kopie") && d.info.is_some_and(|(n, _)| n > 0)); d.ort = ort.clone();
        let akt = |app: &mut App, a: KopAkt| { let _ = app.update(Msg::Kopieren(a)); };
        akt(&mut app, KopAkt::Name("bad/name".into())); assert!(app.kopier.as_ref().unwrap().problem().is_some(), "ungültiger Name"); akt(&mut app, KopAkt::Ok); assert!(!app.kopier.as_ref().unwrap().laeuft, "gesperrt");
        akt(&mut app, KopAkt::Name("Kopie A".into())); assert!(app.kopier.as_ref().unwrap().problem().is_none());
        // der Hintergrundlauf ist ein Task; im Test rechnen wir ihn selbst und liefern das Ergebnis
        let d = app.kopier.as_ref().unwrap(); let rep = vc_ide::copyproject::copy_project(&d.quelle, &d.ort, d.name.trim(), &d.optionen()); assert!(rep.is_ok(), "{rep:?}");
        assert!(ort.join("Kopie A").is_dir() && std::fs::read_dir(ort.join("Kopie A")).unwrap().flatten().any(|f| f.path().extension().is_some_and(|e| e == "vcprj")), "Projektdatei kopiert");
        akt(&mut app, KopAkt::Name("Kopie A".into())); assert!(app.kopier.as_ref().unwrap().problem().is_some_and(|p| p.contains("gibt es schon")), "Name vergeben");
        akt(&mut app, KopAkt::Danach(false)); akt(&mut app, KopAkt::Fertig(rep, false)); assert!(app.kopier.is_none() && app.log.iter().any(|l| l.starts_with("Projekt kopiert")));
        // Abbrechen
        let _ = app.befehl(Cmd::ProjektKopieren); akt(&mut app, KopAkt::Abbruch); assert!(app.kopier.is_none());
        let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&ort); let _ = std::fs::remove_dir_all(&_tmp0);
    }
    #[test] fn quellgruppen_reihenfolge_gruppe_ab_folgt_in_projektdatei_und_baureihenfolge_leere_gruppe_hat_hinweis() {
        use projeinst::PeAkt;
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone();
        std::fs::write(tmp.join("zweite.asm"), "  * = $0801\n  rts\n").unwrap();
        { let datei = tmp.join("sprite_demo.vcprj"); let mut pf = vc_ide::projfile::ProjectFile::load(&datei).unwrap(); pf.sources.push(vc_ide::projfile::Source { targets: vec!["c64".into()], kind: "asm".into(), files: vec!["zweite.asm".into()], ..Default::default() }); pf.save(&datei).unwrap(); }
        app.projekt_neu_laden(); let _ = app.befehl(Cmd::ProjektEinstellungen);
        let akt = |app: &mut App, a: PeAkt| { let _ = app.update(Msg::ProjEinst(a)); };
        let reihenfolge = |root: &std::path::Path, data: &std::path::Path| -> Vec<String> { let pf = vc_ide::projfile::ProjectFile::load(&root.join("sprite_demo.vcprj")).unwrap(); let r = vc_ide::projfile::build_project(&vc_ide::build::Ctx { root, data }, &pf, None); r["results"].as_array().unwrap().iter().map(|x| x["file"].as_str().unwrap().to_string()).collect() };
        assert_eq!(reihenfolge(&tmp, &app.data), ["sprite_demo.asm", "zweite.asm"]);
        // erste Gruppe nach unten: Entwurf, Speichern, Projektdatei und Bauen folgen
        akt(&mut app, PeAkt::GruppeAb(0)); assert_eq!(app.pe.as_ref().unwrap().entwurf.sources[1].files, ["sprite_demo.asm"]); akt(&mut app, PeAkt::GruppeAb(1)); akt(&mut app, PeAkt::GruppeAuf(0)); assert_eq!(app.pe.as_ref().unwrap().entwurf.sources[0].files, ["zweite.asm"], "Auf an der ersten und Ab an der letzten Stelle ändern nichts");
        akt(&mut app, PeAkt::Speichern(false)); assert!(!app.pe.as_ref().unwrap().geaendert(), "{:?}", app.pe.as_ref().unwrap().note);
        let pf = vc_ide::projfile::ProjectFile::load(&tmp.join("sprite_demo.vcprj")).unwrap(); assert_eq!(pf.sources.iter().map(|s| s.files[0].clone()).collect::<Vec<_>>(), ["zweite.asm", "sprite_demo.asm"]);
        assert_eq!(reihenfolge(&tmp, &app.data), ["zweite.asm", "sprite_demo.asm"], "Baureihenfolge folgt der Gruppenreihenfolge");
        // leere Gruppe: Hinweis und Auf/Ab sichtbar
        akt(&mut app, PeAkt::GruppePlus); akt(&mut app, PeAkt::Reiter(projeinst::PeReiter::Quellen)); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text.starts_with("Noch keine Dateien: unten eine hinzufügen")), "Hinweis in leerer Gruppe"); assert!(k.iter().any(|k| k.text == "Gruppe auf") && k.iter().any(|k| k.text == "Gruppe ab"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn bauen_aus_der_oberflaeche_erzeugt_die_diskette_wie_vcc_und_meldet_fehler_sichtbar() {
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone();
        let datei = tmp.join("sprite_demo.vcprj"); let mut pf = vc_ide::projfile::ProjectFile::load(&datei).unwrap();
        pf.disk = Some(vc_ide::projfile::DiskSpec { format: "d64".into(), name: "huibuh".into(), file: "build/Diskette.d64".into(), files: vec!["sprite_demo.asm".into()], ..Default::default() }); pf.save(&datei).unwrap();
        app.projekt_neu_laden(); assert!(app.proj.pf.as_ref().unwrap().disk.is_some());
        let r = vc_ide::projfile::build_gui(&vc_ide::build::Ctx { root: &tmp, data: &app.data }, app.proj.pf.as_ref(), "sprite_demo.asm"); assert!(app.bau_ende(r), "{:?}", app.log);
        assert!(tmp.join("build/Diskette.d64").is_file(), "Diskettenabbild entsteht beim Bauen aus der Oberfläche"); assert!(app.log.iter().any(|l| l.starts_with("Disk build/Diskette.d64") && l.contains("SPRITE_DEMO")), "{:?}", app.log);
        // Fehler sichtbar: Diskette nennt ein Programm, das es nicht gibt
        let mut pf = app.proj.pf.clone().unwrap(); pf.disk.as_mut().unwrap().files = vec!["gibtsnicht.asm".into()]; pf.save(&datei).unwrap(); app.projekt_neu_laden(); app.log.clear();
        let r = vc_ide::projfile::build_gui(&vc_ide::build::Ctx { root: &tmp, data: &app.data }, app.proj.pf.as_ref(), "sprite_demo.asm"); let _ = app.bau_ende(r); assert!(app.log.iter().any(|l| l.starts_with("Diskette:")), "Fehler der Diskette steht im Protokoll: {:?}", app.log);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn projekt_einstellungen_pflichtfelder_leerer_name_ohne_target_sperren_speichern_fehlende_dateien_nur_hinweis() {
        use projeinst::PeAkt;
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); let _ = app.befehl(Cmd::ProjektEinstellungen);
        let datei = tmp.join("sprite_demo.vcprj"); let vorher = std::fs::read(&datei).unwrap();
        let akt = |app: &mut App, a: PeAkt| { let _ = app.update(Msg::ProjEinst(a)); };
        for leer in ["", "   "] {
            akt(&mut app, PeAkt::Name(leer.into())); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text.starts_with("Achtung: Projektname fehlt")), "rote Achtung-Zeile bei Name {leer:?}");
            akt(&mut app, PeAkt::Speichern(false)); assert!(app.pe.as_ref().unwrap().note.starts_with("Nicht gespeichert"), "{:?}", app.pe.as_ref().unwrap().note); assert_eq!(std::fs::read(&datei).unwrap(), vorher, "Datei unverändert");
            akt(&mut app, PeAkt::Speichern(true)); assert_eq!(std::fs::read(&datei).unwrap(), vorher, "auch Speichern und Bauen sperrt");
        }
        akt(&mut app, PeAkt::Name("Neu".into())); let (f, w) = app.pe.as_ref().unwrap().entwurf.validate_getrennt(&tmp); assert!(f.is_empty() && w.is_empty(), "{f:?} {w:?}");
        // fehlende Quelldatei ist nur ein Hinweis: Speichern bleibt möglich
        std::fs::remove_file(tmp.join("sprite_demo.asm")).unwrap(); let (f, w) = app.pe.as_ref().unwrap().entwurf.validate_getrennt(&tmp); assert!(f.is_empty() && w.iter().any(|x| x.contains("Datei fehlt")), "{f:?} {w:?}");
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text.starts_with("Hinweis: sprite_demo.asm: Datei fehlt")));
        akt(&mut app, PeAkt::Speichern(false)); assert!(!app.pe.as_ref().unwrap().geaendert(), "{:?}", app.pe.as_ref().unwrap().note);
        // ohne Target: Fehler
        let mut pf = app.pe.as_ref().unwrap().entwurf.clone(); pf.targets.clear(); assert!(pf.validate_getrennt(&tmp).0.iter().any(|x| x.starts_with("Kein Target")));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn projekt_einstellungen_haben_unterreiter_mit_zaehlern_und_uebersicht_die_diskette_ist_nicht_versteckt() {
        use projeinst::{PeAkt, PeReiter};
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); let _ = app.befehl(Cmd::ProjektEinstellungen);
        let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect() };
        let t = texte(&mut app); for n in ["Allgemein", "Targets 1", "Quellen 1", "Diskette aus", "Übersicht"] { assert!(t.iter().any(|x| x == n), "{n} sichtbar: {t:?}") }
        assert!(t.iter().any(|x| x == "Projektname") || t.iter().any(|x| x == "Name"), "Reiter Allgemein zeigt seine Felder"); assert!(!t.iter().any(|x| x.starts_with("Beim Projektbau ein Diskettenabbild")), "Diskette steht nicht mehr unten auf derselben Seite");
        let _ = app.update(Msg::ProjEinst(PeAkt::Reiter(PeReiter::Diskette))); let t = texte(&mut app); assert!(t.iter().any(|x| x.starts_with("Beim Projektbau ein Diskettenabbild")), "{t:?}"); assert!(!t.iter().any(|x| x == "Videonorm"));
        let _ = app.update(Msg::ProjEinst(PeAkt::DiskAn(true))); let t = texte(&mut app); assert!(t.iter().any(|x| x == "Diskette an"), "Zähler folgt dem Zustand: {t:?}"); assert!(t.iter().any(|x| x.starts_with("d64 ·")), "Übersicht nennt die Diskette: {t:?}");
        let _ = app.update(Msg::ProjEinst(PeAkt::Reiter(PeReiter::Targets))); assert!(texte(&mut app).iter().any(|x| x == "Videonorm"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn projekt_einstellungen_entwurf_bearbeiten_speichern_zuruecksetzen_nur_in_der_kopie() {
        use projeinst::PeAkt;
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); app.root = tmp.clone(); app.projekt_neu_laden(); assert!(app.proj.pf.is_some());
        let _ = app.befehl(Cmd::ProjektEinstellungen); assert_eq!(app.tabs.get(app.aktiv), Some(&Tab::Projekt)); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Projekt-Einstellungen") && k.iter().any(|k| k.text.starts_with("Targets")) && k.iter().any(|k| k.text.starts_with("Quellen")));
        let _ = knoten_sammeln(&mut app, 720, 800);
        let akt = |app: &mut App, a: PeAkt| { let _ = app.update(Msg::ProjEinst(a)); };
        let name0 = app.pe.as_ref().unwrap().entwurf.project.name.clone();
        akt(&mut app, PeAkt::Name("Geändert".into())); akt(&mut app, PeAkt::Beschr("Test".into())); assert!(app.pe.as_ref().unwrap().geaendert()); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "● ungespeichert"));
        akt(&mut app, PeAkt::Zurueck); assert_eq!(app.pe.as_ref().unwrap().entwurf.project.name, name0); assert!(!app.pe.as_ref().unwrap().geaendert());
        // Targets, Quellen, Diskette
        akt(&mut app, PeAkt::TargetNorm(0, "ntsc".into())); assert_eq!(app.pe.as_ref().unwrap().entwurf.targets[0].variant, "ntsc");
        akt(&mut app, PeAkt::GruppePlus); let ng = app.pe.as_ref().unwrap().entwurf.sources.len(); akt(&mut app, PeAkt::GruppeArt(ng - 1, "basic".into())); akt(&mut app, PeAkt::ConvAn(ng - 1, true)); akt(&mut app, PeAkt::ConvFlag(ng - 1, "no_rem", true)); assert!(app.pe.as_ref().unwrap().entwurf.sources[ng - 1].convert.as_ref().unwrap().no_rem); let _ = knoten_sammeln(&mut app, 1280, 800); akt(&mut app, PeAkt::GruppeWeg(ng - 1));
        akt(&mut app, PeAkt::DiskAn(true)); akt(&mut app, PeAkt::DiskFormat("d71".into())); assert_eq!(app.pe.as_ref().unwrap().entwurf.disk.as_ref().unwrap().format, "d71"); let _ = knoten_sammeln(&mut app, 1280, 800);
        // Speichern schreibt die Projektdatei der Kopie
        let datei = tmp.join(app.proj.file.clone().unwrap()); akt(&mut app, PeAkt::Name("Geändert".into())); akt(&mut app, PeAkt::Speichern(false)); assert!(!app.pe.as_ref().unwrap().geaendert(), "{:?}", app.pe.as_ref().unwrap().note);
        let pf = vc_ide::projfile::ProjectFile::load(&datei).unwrap(); assert_eq!(pf.project.name, "Geändert"); assert_eq!(pf.targets[0].variant, "ntsc"); assert_eq!(pf.disk.as_ref().unwrap().format, "d71");
        // Beenden fragt nicht wegen des gespeicherten Entwurfs; ungespeichert zählt als Änderung des Reiters
        akt(&mut app, PeAkt::Name("noch anders".into())); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "● ungespeichert"));
        let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&_t0);
    }
    #[test] fn nur_muster_spielt_das_gewaehlte_muster_zeigt_es_an_und_endet_danach() {
        use songstudio::GAkt;
        let (mut app, tmp) = song_app(); let rel = "music/tune.vcsong"; let _ = simuliere(&mut app, &[]);
        let anzahl = app.songs[rel].doc.anzahl_muster(); assert!(anzahl >= 2, "Beispiel hat mehrere Muster");
        for muster in 0..anzahl {
            let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Stopp)); let _ = aussen::aufgezeichnet();
            let _ = app.update(Msg::SongAkt(rel.into(), GAkt::Muster(muster))); assert_eq!(app.songs[rel].doc.muster, muster);
            // Planung: der Ablauf besteht aus genau diesem Muster, ohne Wiederholung
            let daten = app.songs[rel].doc.song_daten(true); assert_eq!(daten.order, vec![muster], "Ablauf = gewähltes Muster"); assert_eq!(daten.loop_at, usize::MAX, "keine Wiederholung");
            let _ = app.update(Msg::SongAkt(rel.into(), GAkt::SpielMuster)); assert!(app.audio.spielt()); assert_eq!(app.songs[rel].spiel_muster, Some(muster));
            let zeilen = app.songs[rel].doc.zeilen(); let row_secs = daten.row_secs;
            // mitten im Muster: die Anzeige bleibt beim gewählten Muster (nicht beim ersten Eintrag der Reihenfolge)
            app.audio.schritt((row_secs * 44100.0 * (zeilen as f32 / 2.0)) as usize); let _ = app.update(Msg::SongTick);
            assert_eq!(app.songs[rel].doc.muster, muster, "angezeigtes Muster bleibt {muster}"); let (_, z) = app.songs[rel].spiel.unwrap(); assert!(z > 0, "Zeile läuft: {z}");
            let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.art == "text" && k.text.starts_with(&format!("spielt: Muster {muster:02X}"))), "Statuszeile nennt das Muster");
            // nach einem Durchlauf endet die Wiedergabe von selbst
            app.audio.schritt((row_secs * 44100.0 * (zeilen as f32 + 4.0)) as usize); let _ = app.update(Msg::SongTick); assert!(!app.audio.spielt(), "endet nach einem Durchlauf (Muster {muster})"); assert!(app.audio_song.is_none());
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn assistent_wege_entwurf_zusammenfassung_uebernehmen_rueckgaengig_und_abbrechen_nur_in_temp() {
        use assistent::{Erstes, WizAkt}; use vc_ide::setup::{Mode, Part, Topic};
        let (mut app, tmp) = studio_app(); let akt = |app: &mut App, a: WizAkt| { let _ = app.update(Msg::Wiz(a)); };
        let _ = app.befehl(Cmd::Assistent); let w = app.wiz.as_ref().expect("Assistent offen"); assert_eq!(w.thema(), Topic::Welcome); assert!(w.paths.settings.starts_with(std::env::temp_dir()), "im Test nie die echte Konfiguration");
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Schnelleinrichtung starten") && k.iter().any(|k| k.text == "Willkommen bei VintageCoder"));
        let _ = knoten_sammeln(&mut app, 720, 800);
        akt(&mut app, WizAkt::Weg(Mode::Quick)); assert_eq!(app.wiz.as_ref().unwrap().thema(), Topic::Design); assert_eq!(app.wiz.as_ref().unwrap().schritte.len(), 7);
        // Design wirkt sofort, Abbrechen stellt zurück
        let vorher = (app.einst.schema, app.einst.zoom); akt(&mut app, WizAkt::Schema("light".into())); akt(&mut app, WizAkt::Zoom(1.3)); assert_eq!(app.einst.schema, einstellungen::Schema::Hell); assert!((app.einst.zoom - 1.3).abs() < 0.01);
        // Schritte durchlaufen, jeder Schritt zeichnet
        for _ in 0..6 { let _ = knoten_sammeln(&mut app, 1280, 800); let t = app.wiz.as_ref().unwrap().thema(); if t == Topic::Summary { break } akt(&mut app, WizAkt::Weiter); }
        akt(&mut app, WizAkt::Springe(1)); assert_eq!(app.wiz.as_ref().unwrap().thema(), Topic::Design); for _ in 0..4 { akt(&mut app, WizAkt::Weiter) }
        // Systeme wählen
        akt(&mut app, WizAkt::Springe(2)); assert_eq!(app.wiz.as_ref().unwrap().thema(), Topic::Systems); akt(&mut app, WizAkt::System("mega65".into(), true)); assert!(app.satz.entwurf.target("mega65").is_some());
        akt(&mut app, WizAkt::ProjekteOrdner(tmp.join("proj").display().to_string())); akt(&mut app, WizAkt::Erstes(Erstes::Spaeter));
        akt(&mut app, WizAkt::Springe(5)); assert_eq!(app.wiz.as_ref().unwrap().thema(), Topic::Summary); let w = app.wiz.as_ref().unwrap(); let ch = w.aenderungen(); assert!(ch.iter().any(|c| c.key == "target:mega65" && c.selected), "{:?}", ch.iter().map(|c| &c.key).collect::<Vec<_>>());
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Zusammenfassung") && k.iter().any(|k| k.text == "Übernehmen"));
        // eine Änderung abwählen, dann übernehmen: nur Testdateien werden geschrieben
        akt(&mut app, WizAkt::Haken("projects_dir".into(), false)); akt(&mut app, WizAkt::Uebernehmen);
        let w = app.wiz.as_ref().unwrap(); assert!(w.fehler.is_none(), "{:?}", w.fehler); assert!(w.angewandt.is_some()); assert_eq!(w.thema(), Topic::Done); let paths = w.paths.clone();
        let t = std::fs::read_to_string(&paths.settings).unwrap(); assert!(t.contains("mega65") && !t.contains("projects_dir"), "{t}"); assert!(paths.wizard.is_file() && std::fs::read_to_string(&paths.wizard).unwrap().contains("first_run_done"));
        let _ = knoten_sammeln(&mut app, 1280, 800);
        // Rückgängig stellt die Datei her (hier: keine Sicherung, da es sie vorher nicht gab → Datei entfernt)
        akt(&mut app, WizAkt::Rueckgaengig); assert!(app.wiz.as_ref().unwrap().angewandt.is_none() && !std::fs::read_to_string(&paths.settings).unwrap_or_default().contains("mega65"), "rückgängig");
        // Abbrechen: Darstellung zurück, nichts geschrieben
        akt(&mut app, WizAkt::Weg(Mode::Part(Part::DesignEditor))); akt(&mut app, WizAkt::Schema("light".into())); akt(&mut app, WizAkt::Abbruch); assert!(app.wiz.is_none()); assert_eq!((app.einst.schema, app.einst.zoom), vorher, "Abbrechen stellt die Darstellung von vor dem Assistenten wieder her");
        // Hinweis auf der Startseite: unvollständige Einrichtung (offenes Thema) und Ausblenden
        let mut zst = vc_ide::setup::WizardState::default(); zst.mark_later(Topic::Roms); zst.save(&paths.wizard).unwrap(); assert_eq!(app.hinweis_ermitteln(), Some(startui::Hinweis::Unvollstaendig)); app.start.banner = app.hinweis_ermitteln();
        app.tabs.clear(); app.aktiv = 0; let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Fertig einrichten"), "Banner sichtbar");
        let _ = app.update(Msg::StartSeite(startui::StartAkt::BannerWeg)); assert!(app.start.banner.is_none() && app.hinweis_ermitteln().is_none(), "ausgeblendet");
        for f in [&paths.settings, &paths.view, &paths.wizard] { let _ = std::fs::remove_file(f); let _ = std::fs::remove_file(f.with_extension("json.bak")); }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    fn song12_app() -> (App, std::path::PathBuf) { let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/mega65/music_demo"); einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("music/tune4.vcsong"); (app, tmp) }
    #[test] fn tracker_raster_mit_12_kanaelen_scrollt_per_rad_horizontal_und_vertikal() {
        use iced::{mouse, Event, Point}; use iced::keyboard::{key::Named, Key, Modifiers};
        let (mut app, tmp) = song12_app(); let rel = "music/tune4.vcsong"; assert_eq!(app.songs[rel].doc.kanaele, 12); let _ = simuliere(&mut app, &[]);
        let versatz = |app: &mut App| -> Option<(iced::Vector, iced::Rectangle)> { let k = knoten_mit_zustand(app); k.iter().filter(|k| k.art == "scroll" && k.grenzen.width > 150.0 && k.grenzen.y > 100.0).max_by(|a, b| (a.grenzen.width * a.grenzen.height).total_cmp(&(b.grenzen.width * b.grenzen.height))).map(|k| (k.versatz, k.grenzen)) };
        let (v0, g) = versatz(&mut app).expect("Raster-Scrollbereich"); assert!(g.width < 12.0 * app.songs[rel].kb.get() + 34.0, "Raster ist schmaler als der Inhalt: {g:?}");
        let mitte = Point::new(g.x + g.width / 2.0, g.y + g.height / 2.0);
        let rad = |app: &mut App, dx: f32, dy: f32| { let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: mitte }), mitte), (Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Pixels { x: dx, y: dy } }), mitte)]); };
        rad(&mut app, -120.0, 0.0); let (v1, _) = versatz(&mut app).unwrap(); assert!(v1.x > v0.x, "horizontales Rad scrollt: {v0:?} → {v1:?}");
        rad(&mut app, 0.0, -80.0); let (v2, _) = versatz(&mut app).unwrap(); assert!(v2.y > v1.y, "vertikales Rad scrollt: {v1:?} → {v2:?}");
        // Viewport melden (erster Redraw), dann: Cursor mit Pfeiltasten aus dem Sichtbereich bewegen → Ausschnitt führt nach
        let _ = simuliere(&mut app, &[(Event::Window(iced::window::Event::RedrawRequested(std::time::Instant::now())), mitte)]);
        let (ox, _, w, h) = app.songs[rel].ansicht; assert!(w > 100.0 && h > 100.0, "Viewport gemeldet: {:?}", app.songs[rel].ansicht);
        let taste = |app: &mut App, n: Named, mods: Modifiers| { let _ = app.update(Msg::Taste(Key::Named(n), mods)); };
        let zelle_sichtbar = |app: &App| { let s = &app.songs[rel]; let (ox, oy, w, h) = s.ansicht; let (r, c) = s.doc.cursor; let kb = s.kb.get(); let (x0, x1) = (34.0 + c as f32 * kb, 34.0 + (c + 1) as f32 * kb); let (y0, y1) = (r as f32 * 20.0, (r + 1) as f32 * 20.0); x0 >= ox + 34.0 - 0.5 && x1 <= ox + w + 0.5 && y0 >= oy - 0.5 && y1 <= oy + h + 0.5 };
        for _ in 0..11 { taste(&mut app, Named::ArrowRight, Modifiers::empty()); assert!(zelle_sichtbar(&app), "Cursor Kanal {} sichtbar, Ausschnitt {:?}", app.songs[rel].doc.cursor.1, app.songs[rel].ansicht) }
        assert_eq!(app.songs[rel].doc.cursor.1, 11); assert!(app.songs[rel].ansicht.0 > ox + 1.0, "horizontal nachgeführt: {:?}", app.songs[rel].ansicht);
        for _ in 0..31 { taste(&mut app, Named::ArrowDown, Modifiers::empty()); assert!(zelle_sichtbar(&app), "Zeile {} sichtbar", app.songs[rel].doc.cursor.0) }
        assert!(app.songs[rel].ansicht.1 > 0.0, "vertikal nachgeführt: {:?}", app.songs[rel].ansicht);
        for _ in 0..31 { taste(&mut app, Named::ArrowUp, Modifiers::empty()) } for _ in 0..11 { taste(&mut app, Named::ArrowLeft, Modifiers::empty()) } assert!(zelle_sichtbar(&app) && app.songs[rel].ansicht.0 < 1.0 && app.songs[rel].ansicht.1 < 1.0, "zurück am Anfang: {:?}", app.songs[rel].ansicht);
        // Tab springt zum nächsten Kanal und führt ebenfalls nach
        for _ in 0..11 { taste(&mut app, Named::Tab, Modifiers::empty()); assert!(zelle_sichtbar(&app)) }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn projekt_oeffnen_per_projektdatei_wechselt_genau_einmal_und_der_dialog_nimmt_dateien_an() {
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); let vcprj = tmp.join("sprite_demo.vcprj"); assert!(vcprj.is_file());
        // der Öffnen-Befehl ist ein Dateidialog (pick_file mit Filter .vcprj), der Ordner-Dialog ein eigener Befehl
        assert!(befehle::BEFEHLE.iter().any(|i| i.cmd == Cmd::ProjektOrdnerOeffnen && i.name.contains("Projektordner")));
        let alt = app.root.clone(); let _ = app.update(Msg::Geoeffnet(Some(vcprj.clone()))); assert_eq!(app.root, tmp, "Projektdatei öffnet das Projekt"); assert_ne!(app.root, alt); assert_eq!(app.proj.file.as_deref(), Some("sprite_demo.vcprj"));
        // genau einmal: dieselbe Datei nochmal wählen lädt nicht neu (Reiter/Zustand bleiben)
        let tabs = app.tabs.clone(); let lsp = app.lsp_gen; let _ = app.update(Msg::Geoeffnet(Some(vcprj.clone()))); assert_eq!(app.lsp_gen, lsp, "kein zweiter Projektwechsel"); assert_eq!(app.root, tmp); assert!(app.tabs.len() >= tabs.len() - 1);
        let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&_t0);
    }
    #[test] fn ueber_feedback_und_update_nur_ueber_aussen_und_ohne_netz() {
        use infodlg::InfoAkt; use vc_ide::update::{self, Manifest, Report, Version};
        let (mut app, tmp) = studio_app(); let akt = |app: &mut App, a: InfoAkt| { let _ = app.update(Msg::Info(a)); }; let _ = aussen::aufgezeichnet();
        // Über: Dialog, Links nur aufgezeichnet, Drittanbieter-Hinweise
        let _ = app.befehl(Cmd::Ueber); assert!(app.upd.ueber); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.art == "text" && k.text.starts_with("VintageCoder ")) && k.iter().any(|k| k.text == "Feedback senden …"));
        let _ = app.update(Msg::Info(infodlg::InfoAkt::Herkunft)); let k = knoten_sammeln(&mut app, 1280, 1000); assert!(k.iter().any(|k| k.text == "Eigene Zeichensätze (VC-Standard)") && k.iter().any(|k| k.text.contains("CC0-1.0 · Rev.")), "Herkunftstabelle sichtbar"); let _ = app.update(Msg::Info(infodlg::InfoAkt::Herkunft));
        akt(&mut app, InfoAkt::Web(infodlg::WEBSITE)); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::UrlOeffnen(infodlg::WEBSITE.into()))); akt(&mut app, InfoAkt::Hinweise); let _ = knoten_sammeln(&mut app, 720, 800); akt(&mut app, InfoAkt::UeberZu); assert!(!app.upd.ueber);
        // Feedback: Mail wird gebildet, Mailprogramm nur aufgezeichnet, nichts gesendet
        akt(&mut app, InfoAkt::FeedbackAuf); let (m, _) = app.upd.feedback.as_ref().expect("Mail"); assert!(m.body.contains("Zielsystem") && m.url.starts_with("mailto:vintagecoder@drueeke.de"));
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Im Mailprogramm öffnen")); akt(&mut app, InfoAkt::Mail); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::MailOeffnen)); assert!(app.upd.feedback.as_ref().unwrap().1.as_ref().is_some_and(|s| s.0));
        akt(&mut app, InfoAkt::DebugLog(true)); assert!(app.satz.gespeichert.debug_log, "Debug-Protokoll nur in der Testdatei"); akt(&mut app, InfoAkt::FeedbackZu); assert!(app.upd.feedback.is_none());
        // Update: Prüfung nur über aussen; Antwort „neue Version“ → Dialog mit Änderungen, Statusleisten-Hinweis, Link fest aus dem Programm
        let json = r###"{"format":1,"sequence":9,"channel":"stable","published":"2099-01-01T09:00:00Z","latest":"99.0.0","releases":[{"version":"99.0.0","title":"99.0.0 (Test)","notes":"## Neu\n- ein Test"}]}"###;
        let ev = update::evaluate(&Manifest::parse(json).unwrap(), Version::installed()); assert!(ev.update.is_some());
        let r = aussen::update_pruefen("test://", Version::installed()); assert!(r.result.is_err(), "ohne Vorgabe: offline, kein Netz"); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::UpdatePruefen));
        akt(&mut app, InfoAkt::Ergebnis(Box::new(Report { at: update::now_secs(), source: "test://".into(), result: Ok(ev), raw: Some(json.into()) }))); assert!(app.upd.update_offen && app.update_hinweis().is_some());
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Neue Version 99.0.0") && k.iter().any(|k| k.text == "Diese Version überspringen"));
        let _ = aussen::aufgezeichnet(); akt(&mut app, InfoAkt::Ansehen); assert!(aussen::aufgezeichnet().contains(&aussen::Aufruf::UrlOeffnen(update::release_url(Some(Version(99, 0, 0)))))); 
        akt(&mut app, InfoAkt::Spaeter); assert!(!app.upd.update_offen && app.update_hinweis().is_none(), "Ruhezeit blendet den Hinweis aus");
        app.upd.stand.snooze_until = 0; assert!(app.update_hinweis().is_some()); akt(&mut app, InfoAkt::Ueberspringen); assert_eq!(app.satz.gespeichert.updates.skipped, "99.0.0"); assert!(app.update_hinweis().is_none(), "übersprungene Version meldet sich nicht mehr");
        // Fehlerfall: Fehlertext im Über-Dialog statt Absturz
        akt(&mut app, InfoAkt::Ergebnis(Box::new(Report { at: 1, source: "test://".into(), result: Err("offline".into()), raw: None }))); assert!(app.upd.stand.last_result.contains("Fehler"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn alte_kachelkarten_zeigen_eine_klare_meldung_und_kachelsaetze_oeffnen_im_studio() {
        let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/mega65/jumprun"); app.root = tmp.clone(); app.projekt_neu_laden();
        // Öffnen einer älteren Kachelkarte: Rückfrage „Ältere Kachelkarte umwandeln?“ (nie stumm als Text); Abbrechen ändert nichts
        let vorher = std::fs::read(tmp.join("graphics/lv1.vctiles")).unwrap();
        app.oeffne("graphics/lv1.vctiles"); assert_eq!(app.rueckfrage, Some(Rueckfrage::Kachelkarte("graphics/lv1.vctiles".into())), "Rückfrage statt Textöffnen"); assert!(!app.tilesets.contains_key("graphics/lv1.vctiles") && !app.editoren.contains_key("graphics/lv1.vctiles"));
        let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().map(|k| k.text).collect(); assert!(t.iter().any(|x| x == "Ältere Kachelkarte umwandeln?") && t.iter().any(|x| x == "Umwandeln"), "{t:?}");
        let _ = app.update(Msg::RueckfrageNein); assert!(app.rueckfrage.is_none()); assert_eq!(std::fs::read(tmp.join("graphics/lv1.vctiles")).unwrap(), vorher, "unverändert");
        // Umwandeln: Satz und Bank entstehen, die Bank öffnet im Studio, die alte Datei liegt gesichert unter .vcbackup/
        app.oeffne("graphics/lv1.vctiles"); let _ = app.update(Msg::RueckfrageJa);
        assert!(tmp.join("graphics/lv1_tiles.vctiles").is_file() && tmp.join("graphics/lv1.vcscreen").is_file(), "{:?}", app.log); assert!(!tmp.join("graphics/lv1.vctiles").exists(), "alte Datei ersetzt");
        assert!(app.banken.contains_key("graphics/lv1.vcscreen"), "Bank öffnet im Studio: {:?}", app.log.last());
        let sicher: Vec<_> = std::fs::read_dir(tmp.join(".vcbackup")).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with("kachelkarte-")).collect(); assert_eq!(sicher.len(), 1); assert_eq!(std::fs::read(sicher[0].path().join("graphics/lv1.vctiles")).unwrap(), vorher, "Sicherung enthält das Original");
        let tmp2 = kopie_von_beispiel("examples/c64/jumprun"); app.root = tmp2.clone(); app.projekt_neu_laden(); app.oeffne("graphics/hg.vctiles"); assert!(app.tilesets.contains_key("graphics/hg.vctiles"), "Kachelsatz öffnet im Studio");
        // alle Beispieldateien: Kachelsatz → Studio, Kachelkarte → Meldung (nie stumm als Text)
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples"); let mut n = 0;
        fn suche(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) { for e in std::fs::read_dir(d).into_iter().flatten().flatten() { let p = e.path(); if p.is_dir() { suche(&p, out) } else if p.extension().is_some_and(|x| x == "vctiles") { out.push(p) } } }
        let mut alle = vec![]; suche(&repo, &mut alle);
        for p in alle { let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap(); let k = v["kind"].as_str().unwrap_or(""); assert!(k == "tileset" || k == "tilemap", "{}: unbekannte Art {k}", p.display()); n += 1 }
        assert!(n >= 6, "Beispiele gefunden: {n}");
        let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&tmp2); let _ = std::fs::remove_dir_all(&_t0);
    }
    #[test] fn farbwahl_der_dokumentfarben_ist_rueckgaengig_machbar_zeichenfarbe_ist_werkzeugzustand() {
        use screenstudio::SAkt; use studio::Aktion;
        // Bildschirm-Studio: Hintergrund/Rahmen sind Dokumentdaten (Rückgängig/Wiederholen), die Farbe für neue Zeichen ist Werkzeugzustand
        let (mut app, tmp) = screen_app(); let rel = "graphics/title.vcscreen"; let akt = |app: &mut App, a: SAkt| { let _ = app.update(Msg::ScreenAkt(rel.into(), a)); };
        let _ = simuliere(&mut app, &[]); let bg0 = app.screens[rel].doc.farbe_von("bg", 0); let neu = (bg0 + 3) % 16;
        akt(&mut app, SAkt::Rolle("bg", neu)); assert_eq!(app.screens[rel].doc.farbe_von("bg", 99), neu); let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.screens[rel].doc.farbe_von("bg", 99), bg0, "Cmd+Z nimmt die Hintergrundfarbe zurück"); let _ = app.befehl(Cmd::Wiederholen); assert_eq!(app.screens[rel].doc.farbe_von("bg", 99), neu, "Cmd+Umschalt+Z stellt sie wieder her");
        let f0 = app.screens[rel].doc.farbe; akt(&mut app, SAkt::Farbe((f0 + 1) % 16)); assert_eq!(app.screens[rel].doc.farbe, (f0 + 1) % 16); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Farbe für neue Zeichen") || !app.screens[rel].doc.farb_ram, "Beschriftung nennt, was die Farbe ist");
        let _ = std::fs::remove_dir_all(&tmp);
        // Sprite-Studio: Farbwahl ist ein Rückgängig-Schritt
        let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite"; let _ = simuliere(&mut app, &[]); let k0 = app.studios[rel].doc.farbe("sprite", 99);
        let _ = app.update(Msg::StudioAkt(rel.into(), Aktion::Farbe("sprite".into(), 5))); assert_eq!(app.studios[rel].doc.farbe("sprite", 99), 5); let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.studios[rel].doc.farbe("sprite", 99), k0, "Sprite-Farbe rückgängig");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    fn kachel_app() -> (App, std::path::PathBuf) { let (mut app, _t0) = studio_app(); let tmp = kopie_von_beispiel("examples/c64/gfx_demo"); einstellungen::test_konfig(tmp.join("konf")); app.root = tmp.clone(); app.projekt_neu_laden(); app.oeffne("graphics/level_tiles.vctiles"); assert!(app.tilesets.contains_key("graphics/level_tiles.vctiles")); (app, tmp) }
    #[test] fn kachelsatz_farbmodus_je_zelle_trennt_zeichen_und_farbpinsel_und_fuellstand_steht_oben() {
        use tilestudio::{TAkt, TMaus}; use vc_ide_model::tilesetdoc::PinselModus as PM; use vc_media::tiles::ColorMode;
        let (mut app, tmp) = kachel_app(); let rel = "graphics/level_tiles.vctiles"; let akt = |app: &mut App, a: TAkt| { let _ = app.update(Msg::TileAkt(rel.into(), a)); }; let _ = simuliere(&mut app, &[]);
        let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.art == "text" && k.text == "Füllstand des Zeichensatzes"), "Füllstand mit klarer Überschrift"); let pos = |n: &str| k.iter().find(|k| k.art == "text" && k.text == n).map(|k| k.grenzen.y).unwrap_or(1e9); assert!(pos("Füllstand des Zeichensatzes") < 1e9, "Füllstand in der Seitenspalte (Karte, nach dem Mockup unter Kachel und Eigenschaften)");
        akt(&mut app, TAkt::Farbmodus(ColorMode::Cell)); let k = knoten_sammeln(&mut app, 1280, 800); assert!(k.iter().any(|k| k.text == "Ein Klick in die Kachel setzt:") && k.iter().any(|k| k.text == "Farbe") && k.iter().any(|k| k.text == "Zeichen"), "Modus-Auswahl sichtbar bei „je Zelle“");
        let zelle = |app: &App, z: usize| app.tilesets[rel].doc.ts.tiles[0].cells[z].clone(); let vor = zelle(&app, 0);
        // Farbpinsel: nur die Farbe der Zelle ändert sich
        akt(&mut app, TAkt::Modus(PM::Farbe)); akt(&mut app, TAkt::Pinselfarbe(7)); let _ = app.update(Msg::TileMaus(rel.into(), TMaus::Unten { k: 0, rechts: false })); let _ = app.update(Msg::TileMaus(rel.into(), TMaus::Los));
        let nach = zelle(&app, 0); assert_eq!((nach.code, nach.cs, nach.col), (vor.code, vor.cs, Some(7)), "Farbpinsel ändert nur die Farbe: vorher {vor:?}, nachher {nach:?}");
        akt(&mut app, TAkt::Undo); assert_eq!(zelle(&app, 0), vor, "Cmd+Z nimmt die Farbe zurück");
        // Zeichen-Pinsel: nur das Zeichen, Farbe bleibt
        akt(&mut app, TAkt::Pinselfarbe(7)); akt(&mut app, TAkt::Modus(PM::Beides)); let _ = app.update(Msg::TileMaus(rel.into(), TMaus::Unten { k: 1, rechts: false })); let _ = app.update(Msg::TileMaus(rel.into(), TMaus::Los)); let mit_farbe = zelle(&app, 1); assert_eq!(mit_farbe.col, Some(7));
        akt(&mut app, TAkt::Modus(PM::Zeichen)); akt(&mut app, TAkt::Zeichen(65)); let z = zelle(&app, app.tilesets[rel].doc.cur); assert_eq!(z.code, 65, "Zeichen gesetzt"); let cur = app.tilesets[rel].doc.cur; let farbe_vor = zelle(&app, cur).col; akt(&mut app, TAkt::Pinselfarbe(2)); akt(&mut app, TAkt::Zeichen(66)); assert_eq!(zelle(&app, cur).col, farbe_vor, "Zeichen-Pinsel lässt die Zellfarbe stehen");
        // im Farbpinsel-Modus setzt die Zeichentafel nur den Pinsel, malt aber nicht
        akt(&mut app, TAkt::Modus(PM::Farbe)); let c0 = zelle(&app, cur); akt(&mut app, TAkt::Zeichen(70)); assert_eq!(zelle(&app, cur), c0, "Farbmodus: Tafel malt nicht");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn kachelsatz_eigenschaften_haken_und_neue_eigenschaft_mit_echten_klicks() {
        use tilestudio::TAkt; use iced::{mouse, Event, Point};
        let (mut app, tmp) = kachel_app(); let rel = "graphics/level_tiles.vctiles"; let _ = simuliere(&mut app, &[]);
        let klick = |app: &mut App, p: (f32, f32)| { let p = Point::new(p.0, p.1); let _ = simuliere(app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]); };
        // Haken „solid“ per echtem Klick: Eigenschaft der gewählten Kachel gesetzt, Cmd+Z nimmt sie zurück
        let sel = app.tilesets[rel].doc.sel; let vor = app.tilesets[rel].doc.ts.tiles[sel].props & 1;
        let _ = app.update(Msg::TileAkt(rel.into(), TAkt::Karte("kachel"))); let _ = app.update(Msg::TileAkt(rel.into(), TAkt::Karte("werkzeug"))); // Karten Kachel und Werkzeug zu: Dokument rückt nach oben (Seitenspalte scrollt nur, wenn die Karten nicht passen)
        let (x, y) = ort(&mut app, "solid"); klick(&mut app, (x, y)); let nach = app.tilesets[rel].doc.ts.tiles[sel].props & 1; assert_ne!(nach, vor, "Klick auf den Haken schaltet die Eigenschaft");
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.tilesets[rel].doc.ts.tiles[sel].props & 1, vor, "Cmd+Z nimmt sie zurück");
        // neue Eigenschaft: Name tippen + „+“ (Nachrichten wie das Textfeld sie sendet), Doppelte/Leere abgewiesen, höchstens 8
        let akt = |app: &mut App, a: TAkt| { let _ = app.update(Msg::TileAkt(rel.into(), a)); };
        akt(&mut app, TAkt::PropText("platform".into())); akt(&mut app, TAkt::PropNeu); assert!(app.tilesets[rel].doc.ts.prop_defs.iter().any(|(n, _)| n == "platform"), "neue Eigenschaft angelegt"); assert!(app.tilesets[rel].neue_prop.is_empty(), "Feld geleert");
        akt(&mut app, TAkt::PropText("platform".into())); akt(&mut app, TAkt::PropNeu); assert_eq!(app.tilesets[rel].doc.ts.prop_defs.iter().filter(|(n, _)| n == "platform").count(), 1, "Doppelte abgewiesen");
        akt(&mut app, TAkt::PropText("   ".into())); akt(&mut app, TAkt::PropNeu); let n = app.tilesets[rel].doc.ts.prop_defs.len();
        for i in 0..10 { akt(&mut app, TAkt::PropText(format!("p{i}"))); akt(&mut app, TAkt::PropNeu) } assert_eq!(app.tilesets[rel].doc.ts.prop_defs.len(), 8, "höchstens 8 (vorher {n})");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Jede Funktion der Kopfzeile ist in jeder Fensterbreite erreichbar: als Knopf oder im „⋯“-Überlaufmenü; Starten bleibt immer sichtbar
    #[test] fn kopfzeile_erreichbar_in_jeder_breite() {
        for b in [480u32, 560, 640, 720, 800, 900, 1000, 1100, 1280, 1600] {
            let (mut app, tmp) = galerie_zustaende().into_iter().find(|(n, _)| *n == "editor").map(|(_, f)| f()).unwrap(); app.einst.zoom = 1.0; app.proj.name = "Referenz".into();
            let texte = |app: &mut App| -> Vec<String> { knoten_sammeln(app, b, 800).into_iter().filter(|k| k.art == "text" && k.grenzen.width >= 1.0 && k.grenzen.y < 48.0).map(|k| k.text).collect() };
            let t0 = texte(&mut app); let stufe = app.kopf_stufe();
            let hat = |ts: &[String], z: &str| ts.iter().any(|t| t.contains(z));
            assert!(hat(&t0, "▶"), "{b}: Starten fehlt: {t0:?}"); assert!(hat(&t0, "⚙"), "{b}: Einstellungen fehlen");
            assert!(hat(&t0, "⌕") || hat(&t0, "Suchen") || hat(&t0, t::SUCHFELD), "{b}: Suche fehlt: {t0:?}");
            if stufe < 3 { assert!(hat(&t0, "▣") && !hat(&t0, "⋯") || hat(&t0, "Debuggen") && hat(&t0, "Bauen"), "{b}: Debuggen/Bauen fehlen: {t0:?}") }
            else {
                assert!(hat(&t0, "⋯"), "{b}: Überlaufknopf fehlt");
                let _ = app.update(Msg::KopfMenue); let t1 = knoten_sammeln(&mut app, b, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect::<Vec<_>>();
                assert!(hat(&t1, "Debuggen") && hat(&t1, "Bauen"), "{b}: Menü enthält nicht Debuggen und Bauen: {t1:?}");
            }
            // Projektname gekürzt, nie breiter als der Platz: kein Befund in der Kopfzeile
            let k = knoten_sammeln(&mut app, b, 800); let kopf_befunde: Vec<String> = layout_befunde(&k, b as f32, 800.0, false).into_iter().filter(|f| f.contains("Starten") || f.contains("⚙") || f.contains("Referenz") || f.contains("⌕")).collect();
            assert!(kopf_befunde.is_empty(), "{b}: {kopf_befunde:?}");
            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
    /// Alle Ansichten in drei Fenstergrößen: bekannte Befunde stehen in `tests/layout_bekannt.txt`; neue Befunde schlagen fehl, behobene werden gemeldet.
    /// Datei neu schreiben (nach Freigabe): `VC_UPDATE_REFS=1 cargo test -p vc-gui layout_pruefung`
    #[test] fn layout_pruefung() {
        let ueberlagert = ["dialog-ueber", "dialog-feedback", "assistent-willkommen", "assistent-zusammenfassung", "dialog-neues-projekt", "dialog-projekt-kopieren", "dialog-speichern-unter", "dialog-wiederherstellung", "dialog-beenden", "karte", "baum-menue", "menueleiste", "menueleiste-schmal", "befehlspalette", "dialog-export", "dialog-import", "dialog-neue-datei", "dialog-song-import"];
        let mut alle: Vec<String> = vec![];
        for (name, bauen) in galerie_zustaende() {
            let (mut app, tmp) = bauen(); app.einst.zoom = 1.0; app.einst.schema = Schema::Dunkel; app.proj.name = "Referenz".into(); // fester Name: die Breite der Kopfzeile darf nicht vom Namen des Temp-Ordners abhängen
            for (b, h) in [(1280u32, 800u32), (900, 700), (720, 800)] { let k = knoten_sammeln(&mut app, b, h); assert!(k.len() > 20, "{name}: Widget-Baum ist leer"); for f in layout_befunde(&k, b as f32, h as f32, ueberlagert.contains(&name)) { alle.push(format!("{name} | {b}x{h} | {f}{}", if name == "geteilt" && b < 1000 { " [Entschieden 2026-11-16: unter ~1000 px untereinander teilen, Umsetzung im UX-Paket (Breitenklassen)]" } else { "" })) } }
            let _ = std::fs::remove_dir_all(&tmp);
        }
        alle.sort(); alle.dedup();
        let datei = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/layout_bekannt.txt");
        if std::env::var_os("VC_UPDATE_REFS").is_some() { std::fs::create_dir_all(datei.parent().unwrap()).unwrap(); std::fs::write(&datei, alle.join("\n") + "\n").unwrap(); return }
        let bekannt: std::collections::HashSet<String> = std::fs::read_to_string(&datei).unwrap_or_default().lines().map(String::from).collect();
        let neu: Vec<&String> = alle.iter().filter(|f| !bekannt.contains(*f)).collect(); let behoben = bekannt.iter().filter(|f| !alle.contains(f)).count();
        eprintln!("Layout-Befunde: {} gesamt, {} bekannt, {} neu, {} behoben", alle.len(), alle.len() - neu.len(), neu.len(), behoben);
        assert!(neu.is_empty(), "neue Layout-Befunde:\n{}", neu.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
    }
    // ---------------------------------------------------------------------------------------------- Bildvergleich
    /// Ausgewählte Ansichten gegen Referenzbilder in `tests/ref/` (Dunkel, 1280×800). Fehlt eine Referenz, wird sie übersprungen (Freigabe durch den Projektinhaber nötig).
    /// Bei Abweichung (mehr als 0,5 % der Pixel um mehr als 24 je Kanal) entstehen `build/bildvergleich/<name>-ist.png` und `-diff.png`.
    /// Referenzen neu schreiben (nur nach Freigabe): `VC_UPDATE_REFS=1 cargo test -p vc-gui bildvergleich`
    #[test] fn bildvergleich_mit_referenzbildern() {
        let wahl = ["editor", "studio-sprite", "studio-tracker", "studio-bank", "dialog-export", "debug-angehalten"];
        let (refdir, aus) = (std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ref"), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/bildvergleich"));
        let aktualisieren = std::env::var_os("VC_UPDATE_REFS").is_some(); let (mut verglichen, mut fehler) = (0, vec![]);
        for (name, bauen) in galerie_zustaende() {
            if !wahl.contains(&name) { continue }
            let (mut app, tmp) = bauen(); app.einst.zoom = 1.0; app.einst.schema = Schema::Dunkel; app.proj.name = "Referenz".into();
            std::fs::create_dir_all(&aus).unwrap(); let ist = aus.join(format!("{name}-ist.png")); foto_gross(&mut app, &[], ist.to_str().unwrap(), 1280, 800); let _ = std::fs::remove_dir_all(&tmp);
            let refp = refdir.join(format!("{name}.png"));
            if aktualisieren { std::fs::create_dir_all(&refdir).unwrap(); std::fs::copy(&ist, &refp).unwrap(); continue }
            let Ok(soll) = tiny_skia::Pixmap::load_png(&refp) else { eprintln!("Bildvergleich {name}: keine Referenz (übersprungen)"); continue };
            let ist_bild = tiny_skia::Pixmap::load_png(&ist).unwrap(); verglichen += 1;
            if (soll.width(), soll.height()) != (ist_bild.width(), ist_bild.height()) { fehler.push(format!("{name}: Größe {}×{} statt {}×{}", ist_bild.width(), ist_bild.height(), soll.width(), soll.height())); continue }
            let mut diff = tiny_skia::Pixmap::new(soll.width(), soll.height()).unwrap(); let mut anders = 0usize;
            for (i, (a, b)) in soll.data().chunks_exact(4).zip(ist_bild.data().chunks_exact(4)).enumerate() { let d = (0..3).map(|k| (a[k] as i32 - b[k] as i32).abs()).max().unwrap_or(0); let o = i * 4; if d > 24 { anders += 1; diff.data_mut()[o..o + 4].copy_from_slice(&[255, 0, 255, 255]) } else { diff.data_mut()[o..o + 4].copy_from_slice(&[b[0] / 3, b[1] / 3, b[2] / 3, 255]) } }
            let anteil = anders as f64 / (soll.width() * soll.height()) as f64;
            if anteil > 0.005 { diff.save_png(aus.join(format!("{name}-diff.png"))).unwrap(); fehler.push(format!("{name}: {:.2} % der Pixel weichen ab (Differenzbild: build/bildvergleich/{name}-diff.png)", anteil * 100.0)) }
        }
        eprintln!("Bildvergleich: {verglichen} Bilder verglichen"); assert!(fehler.is_empty(), "Bildabweichungen:\n{}", fehler.join("\n"));
    }
    // ---------------------------------------------------------------------------------------------- Kontrast
    /// Kontrastverhältnis nach WCAG 2.x (relative Leuchtdichte)
    fn kontrast(a: iced::Color, b: iced::Color) -> f32 { let l = |c: iced::Color| { let f = |x: f32| if x <= 0.03928 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }; 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b) }; let (x, y) = (l(a), l(b)); (x.max(y) + 0.05) / (x.min(y) + 0.05) }
    /// Text- und Flächenfarben aller Farbrollen gegen 4,5:1 (normaler Text) in Hell und Dunkel, gegen 7:1 in „Hoher Kontrast“; keine Ausnahmeliste
    #[test] fn farbkontraste_aller_rollen_in_allen_paletten() {
        let mut alle: Vec<String> = vec![];
        for (n, p, grenze) in [("dunkel", Pal::dark(), 4.5f32), ("hell", Pal::light(), 4.5), ("kontrast", Pal::hoch(), 7.0)] {
            let flaechen = [("bg", p.bg), ("panel", p.panel), ("panel2", p.panel2), ("editor", p.editor), ("sel", p.sel)];
            let texte = [("text", p.text), ("mute", p.mute), ("accent", p.accent), ("ok", p.ok), ("warn", p.warn), ("err", p.err), ("lab", p.lab), ("dir", p.dir)];
            for (tn, t) in texte { for (fn_, f) in flaechen { let k = kontrast(t, f); if k < grenze { alle.push(format!("{n}: {tn} auf {fn_} {k:.1}:1 (< {grenze})")) } } }
            for (tn, t) in [("kw", p.kw), ("num", p.num), ("string", p.string), ("cmt", p.cmt)] { let k = kontrast(t, p.editor); if k < grenze { alle.push(format!("{n}: Syntax {tn} auf editor {k:.1}:1 (< {grenze})")) } }
            // Beschriftung auf gefüllten Knöpfen (Hauptknopf: ok-Fläche mit dunkler bzw. weißer Schrift) und auf Akzentflächen
            let auf_haupt = if p.dark { iced::Color::BLACK } else { iced::Color::WHITE }; let k = kontrast(auf_haupt, p.ok); if k < grenze { alle.push(format!("{n}: Hauptknopf-Schrift auf ok {k:.1}:1 (< {grenze})")) }
            let k = kontrast(p.accent_text(), p.accent); if k < grenze { alle.push(format!("{n}: Schrift auf accent {k:.1}:1 (< {grenze})")) }
            // Rahmenlinien müssen als Umriss erkennbar sein (nicht-textliche Elemente: 3:1), im hohen Kontrast deutlich
            if p.hoch { let k = kontrast(p.line, p.panel); if k < 3.0 { alle.push(format!("{n}: Rahmenlinie auf panel {k:.1}:1 (< 3)")) } }
        }
        assert!(alle.is_empty(), "Kontrastbefunde:\n{}", alle.join("\n"));
        assert!(kontrast(iced::Color::BLACK, iced::Color::WHITE) > 20.9 && kontrast(iced::Color::WHITE, iced::Color::WHITE) < 1.01, "Kontrastformel");
    }
    #[test] #[ignore] fn bild_export_dialog() { let (mut app, tmp) = studio_app(); let _ = app.befehl(Cmd::Exportieren); foto(&mut app, &[], "/tmp/vc-export.png"); let _ = std::fs::remove_dir_all(&tmp); }
    fn palette_app() -> (App, std::path::PathBuf) {
        let (mut app, tmp) = studio_app(); let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profile = vc_media::load_profiles(&repo.join("data")).unwrap(); let p = profile.iter().find(|p| p.kind == "palette").unwrap().clone();
        std::fs::write(tmp.join("farben.vcpal"), vc_media::project::format_media(&vc_media::new_media(&p, "farben"))).unwrap(); app.projekt_neu_laden(); app.oeffne("farben.vcpal"); (app, tmp)
    }
    #[test] fn paletten_studio_regler_hexwert_undo_speichern_und_export() {
        { // Muster: Kopfzeile, Leiste, Eintragskarte (Texte), Datei-Aktionen nur in der Kopfzeile
            let (mut app, tmp) = palette_app(); let t: Vec<String> = knoten_sammeln(&mut app, 1280, 800).into_iter().filter(|k| k.art == "text").map(|k| k.text).collect();
            for n in ["PAL", "Exportieren …", "Speichern", "Standardpalette", "Eintrag 0 ($00)"] { assert!(t.iter().any(|x| x == n), "{n} sichtbar: {t:?}") }
            assert!(t.iter().any(|x| x.ends_with("Farben")), "Kopfzeile nennt die Zahl der Farben: {t:?}"); let _ = std::fs::remove_dir_all(&tmp);
        }
        use palettestudio::PAkt; let (mut app, tmp) = palette_app(); let rel = "farben.vcpal"; let akt = |app: &mut App, a: PAkt| { let _ = app.update(Msg::PaletteAkt(rel.into(), a)); };
        assert!(app.paletten.contains_key(rel) && app.name().as_deref() == Some(rel), "Palette öffnet im Paletten-Studio"); let _ = simuliere(&mut app, &[]);
        akt(&mut app, PAkt::Wahl(5)); akt(&mut app, PAkt::Kanal(0, 200)); akt(&mut app, PAkt::Kanal(2, 30)); akt(&mut app, PAkt::Fertig);
        assert_eq!(app.paletten[rel].rgb(5)[0], 200); assert!(app.paletten[rel].dirty()); assert_eq!(app.pal_hex.len(), 7);
        akt(&mut app, PAkt::Hex("#102030".into())); akt(&mut app, PAkt::HexOk); assert_eq!(app.paletten[rel].rgb(5), [16, 32, 48]);
        akt(&mut app, PAkt::Hex("kaputt".into())); akt(&mut app, PAkt::HexOk); assert_eq!(app.paletten[rel].rgb(5), [16, 32, 48], "ungültiger Hexwert ändert nichts"); assert_eq!(app.pal_hex, "#102030");
        let _ = app.befehl(Cmd::Rueckgaengig); assert_eq!(app.paletten[rel].rgb(5)[0], 200, "Cmd+Z macht den Hexwert rückgängig, die Reglerschritte waren ein Schritt");
        let _ = app.befehl(Cmd::Wiederholen); assert_eq!(app.paletten[rel].rgb(5), [16, 32, 48]);
        let _ = app.befehl(Cmd::Speichern); assert!(!app.paletten[rel].dirty());
        let _ = app.befehl(Cmd::Exportieren); let d = app.export.as_ref().expect("Export-Dialog für Paletten"); assert_eq!(d.kind, "palette"); assert!(d.cache.error.is_none() && !d.cache.files.is_empty(), "{:?}", d.cache.error);
        let _ = simuliere(&mut app, &[]); let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Schliessen)); akt(&mut app, PAkt::Standard); assert!(app.paletten[rel].dirty());
        let _ = simuliere(&mut app, &[]); let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_paletten_studio() { let (mut app, tmp) = palette_app(); let _ = app.update(Msg::PaletteAkt("farben.vcpal".into(), palettestudio::PAkt::Wahl(37))); foto(&mut app, &[], "/tmp/vc-palette.png"); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] fn studio_tastatur_mit_echten_ereignissen_auswahl_zwischenablage_loeschen_rueckgaengig() {
        use iced::{keyboard::{key::Named, Event as K, Key, Location, Modifiers, key}, Event, Point}; let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite"; let _ = simuliere(&mut app, &[]);
        let mitte = Point::new(740.0, 380.0);
        let named = |n: Named| Event::Keyboard(K::KeyPressed { key: Key::Named(n.clone()), modified_key: Key::Named(n), physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified), location: Location::Standard, modifiers: Modifiers::empty(), text: None, repeat: false });
        let kombi = |t: &str| kuerzel_ev(t).0;
        let sende = |app: &mut App, e: Event| {
            // wie im Programm: erst die Ansicht (Textfelder, Editor), nur ignorierte Tasten gehen als `Taste` weiter (Abonnement)
            let (msgs, st) = ui_status(app, e.clone(), mitte); for m in msgs { let _ = app.update(m); }
            if let Event::Keyboard(K::KeyPressed { key, modifiers, .. }) = e { if st.iter().all(|s| *s == iced::event::Status::Ignored) { let _ = app.update(Msg::Taste(key, modifiers)); } }
        };
        let (w, h) = { let d = &app.studios[rel].doc; (d.width, d.height) };
        let leer = |app: &App| app.studios[rel].doc.frames[0].iter().all(|z| z.iter().all(|v| *v == 0));
        sende(&mut app, kombi("Cmd+A")); assert_eq!(app.studios[rel].doc.sel, Some((0, 0, w - 1, h - 1)), "Cmd+A wählt alles");
        let vorher = app.studios[rel].doc.frames[0].clone();
        sende(&mut app, kombi("Cmd+C")); assert!(app.studios[rel].doc.clip.is_some() && app.studios[rel].doc.frames[0] == vorher, "Cmd+C kopiert");
        sende(&mut app, kombi("Cmd+X")); assert!(leer(&app), "Cmd+X schneidet aus");
        sende(&mut app, kombi("Cmd+V")); assert_eq!(app.studios[rel].doc.frames[0], vorher, "Cmd+V fügt ein");
        sende(&mut app, kombi("Cmd+Z")); assert!(leer(&app), "Cmd+Z macht das Einfügen rückgängig");
        sende(&mut app, kombi("Cmd+Umschalt+Z")); assert_eq!(app.studios[rel].doc.frames[0], vorher, "Cmd+Umschalt+Z wiederholt");
        sende(&mut app, named(Named::Delete)); assert!(leer(&app), "Entf löscht die Auswahl");
        sende(&mut app, named(Named::Escape)); assert!(app.studios[rel].doc.sel.is_none(), "Esc hebt die Auswahl auf");
        sende(&mut app, kombi("Cmd+Z")); assert_eq!(app.studios[rel].doc.frames[0], vorher, "Cmd+Z macht Entf rückgängig");
        // Menü „Bearbeiten“ (native Menüleiste meldet die Befehlskennung): Kopieren/Ausschneiden/Einfügen wirken im Studio
        sende(&mut app, kombi("Cmd+A")); app.studios.get_mut(rel).unwrap().doc.clip = None;
        let _ = app.update(Msg::Menue("kopieren".into())); assert!(app.studios[rel].doc.clip.is_some(), "Menü Kopieren");
        let _ = app.update(Msg::Menue("ausschneiden".into())); assert!(leer(&app), "Menü Ausschneiden");
        let _ = app.update(Msg::Menue("einfuegen".into())); assert_eq!(app.studios[rel].doc.frames[0], vorher, "Menü Einfügen");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Alt+Buchstabe gehört der Menüleiste: unter macOS liefert Alt+D das Textzeichen „∂“, das der Editor sonst einfügt und das Ereignis verbraucht
    #[test] fn editor_reserviert_alt_buchstaben_fuer_die_menueleiste_nur_wenn_sie_sichtbar_ist() {
        use iced::keyboard::{key, Event as K, Key, Location, Modifiers};
        let ev = |c: &str, text: &str, m: Modifiers| iced::Event::Keyboard(K::KeyPressed { key: Key::Character(c.into()), modified_key: Key::Character(text.into()), physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified), location: Location::Standard, modifiers: m, text: Some(text.into()), repeat: false });
        let (mut app, tmp) = debug_app(); app.dbg = None; einstellungen::test_konfig(tmp.join("konf")); app.kuerzel_windows = true; let _ = simuliere(&mut app, &[]);
        let p = iced::Point::new(330.0 + 8.0 * 7.8, 94.0 + 2.0 * 19.0 + 29.0);
        let klick = |app: &mut App| { let _ = simuliere(app, &[(iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: p }), p), (iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)), p), (iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)), p)]); };
        app.menue_test = true; klick(&mut app);
        let text_vorher = app.ed().unwrap().text();
        // Menüleiste sichtbar: Alt+D (Text „∂“) wird nicht eingefügt und nicht verbraucht
        let (msgs, st) = ui_status(&mut app, ev("d", "∂", Modifiers::ALT), p);
        assert!(st.iter().all(|s| *s == iced::event::Status::Ignored), "Alt+D bleibt für die Menüleiste frei");
        assert!(!msgs.iter().any(|m| matches!(m, Msg::EdIn(_, vc_gui_editor::widget::EdMsg::Insert(_)))), "kein Zeichen eingefügt: {msgs:?}");
        // andere Alt-Zeichen bleiben Text (Option+8 = „{“ auf deutschen Mac-Tastaturen)
        let (m0, _) = ui_status(&mut app, ev("x", "x", Modifiers::empty()), p); assert!(m0.iter().any(|m| matches!(m, Msg::EdIn(..))), "Editor hat den Fokus: {m0:?}");
        let (msgs, st) = ui_status(&mut app, ev("8", "{", Modifiers::ALT), p);
        assert!(msgs.iter().any(|m| matches!(m, Msg::EdIn(_, vc_gui_editor::widget::EdMsg::Insert(t)) if t == "{")) && st.iter().any(|s| *s == iced::event::Status::Captured), "Alt+8 fügt „{{“ ein");
        // AltGr (= Strg+Alt) bleibt unberührt: das Zeichen wird eingefügt oder zumindest nicht für die Menüleiste reserviert (Strg-Fall des Editors ändert sich nicht)
        let (msgs, _) = ui_status(&mut app, ev("q", "@", Modifiers::ALT | Modifiers::CTRL), p);
        assert!(!msgs.iter().any(|m| matches!(m, Msg::EdIn(_, vc_gui_editor::widget::EdMsg::Insert(t)) if t == "∂")));
        // Menüleiste aus: Alt+D ist ein gewöhnliches Textzeichen
        app.menue_test = false; app.einst.menueleiste = false; klick(&mut app);
        let (msgs, st) = ui_status(&mut app, ev("d", "∂", Modifiers::ALT), p);
        assert!(msgs.iter().any(|m| matches!(m, Msg::EdIn(_, vc_gui_editor::widget::EdMsg::Insert(t)) if t == "∂")) && st.iter().any(|s| *s == iced::event::Status::Captured), "ohne Menüleiste wird „∂“ eingefügt");
        // macOS: Option+Buchstabe bleibt immer Text, auch bei sichtbarer Leiste (nichts reserviert)
        if cfg!(target_os = "macos") { app.kuerzel_windows = false; app.menue_test = true; klick(&mut app); let (msgs, _) = ui_status(&mut app, ev("d", "∂", Modifiers::ALT), p); assert!(msgs.iter().any(|m| matches!(m, Msg::EdIn(_, vc_gui_editor::widget::EdMsg::Insert(t)) if t == "∂")), "macOS: ∂ wird eingefügt"); }
        let _ = text_vorher; let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Tracker mit vielen Kanälen (MEGA65, 12 Stimmen): waagerecht scrollen, Cursor bleibt sichtbar; Kanäle im Raster ausblenden (nur Ansicht)
    #[test] fn tracker_12_kanaele_scrollt_waagerecht_und_blendet_kanaele_aus() {
        use iced::keyboard::{key::Named, Key, Modifiers}; use songstudio::GAkt;
        let tmp = kopie_von_beispiel("examples/mega65/music_demo"); einstellungen::test_konfig(tmp.join("konf"));
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); assert!(app.projekt_oeffnen(tmp.clone())); app.oeffne("music/tune4.vcsong"); let rel = "music/tune4.vcsong";
        let _ = knoten_sammeln(&mut app, 1280, 800); assert_eq!(app.songs[rel].doc.kanaele, 12);
        // waagerechtes Scrollen: Cursor in Kanal 12 → der Ausschnitt rückt nach rechts, Kanal 1 in den Cursor zurück → wieder nach links
        let _ = app.update(Msg::SongScroll(rel.into(), 0.0, 0.0, 700.0, 400.0)); app.songs.get_mut(rel).unwrap().doc.cursor.1 = 11;
        let _ = app.song_nachfuehren(); assert!(app.songs[rel].ansicht.0 > 100.0, "nach rechts nachgescrollt: {:?}", app.songs[rel].ansicht);
        app.songs.get_mut(rel).unwrap().doc.cursor.1 = 0; let _ = app.song_nachfuehren(); assert!(app.songs[rel].ansicht.0 < 1.0, "zurück nach links");
        // ausblenden: Kanal 2 und 3 (Index 1, 2), die Datei bleibt unverändert, Wiedergabe-Stummschaltung auch
        let akt = |app: &mut App, a: GAkt| { let _ = app.update(Msg::SongAkt(rel.into(), a)); };
        app.songs.get_mut(rel).unwrap().doc.cursor.1 = 1; akt(&mut app, GAkt::Sichtbar(1)); akt(&mut app, GAkt::Sichtbar(2));
        { let s = &app.songs[rel]; assert_eq!(s.sichtbar().len(), 10); assert_eq!(s.doc.cursor.1, 3, "Cursor springt zum nächsten sichtbaren Kanal"); assert!(!s.doc.dirty(), "ändert die Datei nicht"); assert!(!s.doc.stumm[1], "ausgeblendet ist nicht stumm") }
        // Pfeiltasten überspringen ausgeblendete Kanäle
        app.songs.get_mut(rel).unwrap().doc.cursor.1 = 0; let _ = app.update(Msg::Taste(Key::Named(Named::ArrowRight), Modifiers::empty())); assert_eq!(app.songs[rel].doc.cursor.1, 3, "rechts von Kanal 1 liegt Kanal 4");
        let _ = app.update(Msg::Taste(Key::Named(Named::ArrowLeft), Modifiers::empty())); assert_eq!(app.songs[rel].doc.cursor.1, 0, "links zurück");
        // der Zustand bleibt beim Wechseln des Musters; der letzte sichtbare Kanal lässt sich nicht ausblenden
        akt(&mut app, GAkt::Muster(1)); assert_eq!(app.songs[rel].sichtbar().len(), 10, "Muster wechseln ändert die Ansicht nicht");
        for c in 0..12 { akt(&mut app, GAkt::Sichtbar(c)); if c == 0 { akt(&mut app, GAkt::Sichtbar(c)) } } // erst aus, dann wieder an; danach alle außer einem aus
        assert!(!app.songs[rel].sichtbar().is_empty(), "mindestens ein Kanal bleibt sichtbar");
        // Befehl „Alle Kanäle einblenden“ in Palette und Kontextmenü, nur wenn etwas ausgeblendet ist
        let a = app.studio_aktionen(); let e = a.iter().find(|x| x.name == "Alle Kanäle einblenden").expect("Befehl vorhanden"); assert!(e.aktiv && e.menue, "aktiv, wenn Kanäle fehlen");
        akt(&mut app, GAkt::AlleEinblenden); assert_eq!(app.songs[rel].sichtbar().len(), 12); assert!(!app.studio_aktionen().iter().find(|x| x.name == "Alle Kanäle einblenden").unwrap().aktiv, "ausgegraut, wenn alles sichtbar ist");
        // die Ansicht zeichnet ohne Fehler (auch mit ausgeblendeten Kanälen)
        akt(&mut app, GAkt::Sichtbar(5)); let _ = knoten_sammeln(&mut app, 1280, 800);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn kein_widget_verbraucht_kuerzel_der_befehlstabelle_bei_fokus_im_studio() {
        let (mut app, tmp) = studio_app(); let _ = simuliere(&mut app, &[]); let mitte = iced::Point::new(740.0, 380.0);
        // Fläche anklicken (Fokus im Studio), dann jedes Kürzel der Tabelle als Ereignis: die Ansicht darf es nicht verbrauchen
        let _ = simuliere(&mut app, &[(iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: mitte }), mitte), (iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)), mitte), (iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)), mitte)]);
        for i in befehle::BEFEHLE.iter().filter(|i| !i.tasten.is_empty()) {
            let (ev, _, _) = kuerzel_ev(i.tasten); let (_, st) = ui_status(&mut app, ev, mitte);
            assert!(st.iter().all(|s| *s == iced::event::Status::Ignored), "{} ({}) wird von einem Widget verbraucht", i.name, i.tasten);
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_export_baum() {
        let tmp = kopie_von_beispiel("examples/c64/sprite_demo"); einstellungen::test_konfig(tmp.join("konf")); let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe();
        assert!(app.projekt_oeffnen(tmp.clone())); app.oeffne("graphics/hero.vcsprite"); let _ = app.befehl(Cmd::Exportieren); let _ = app.update(Msg::ExportAus(exportdlg::ExportAus::Speichern)); foto(&mut app, &[], "/tmp/vc-export-baum.png");
    }
    #[test] #[ignore] fn bild_pmg_ship() {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."); let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe();
        einstellungen::test_konfig(std::env::temp_dir().join("vcgui-konf-pmg")); assert!(app.projekt_oeffnen(repo.join("examples/atari/pmg_ship")));
        app.oeffne("graphics/ship.vcpmg"); foto(&mut app, &[], "/tmp/vc-pmg.png");
    }
    #[test] fn atari_farben_im_studio_folgen_der_gtia_palette_und_das_ziel_der_datei() {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."); let data = repo.join("data");
        let pal: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data.join("palettes/atari.json")).unwrap()).unwrap(); let pal_pal = pal["pal"].as_array().unwrap();
        let farben = studio::Farben::laden(&data);
        for v in [0x00usize, 0x0F, 0x34, 0x96, 0xC8, 58, 134, 196, 14] {
            let hex = pal_pal[v / 2].as_str().unwrap(); let erwartet = [u8::from_str_radix(&hex[1..3], 16).unwrap(), u8::from_str_radix(&hex[3..5], 16).unwrap(), u8::from_str_radix(&hex[5..7], 16).unwrap()];
            let c = farben.farbe("atari8", "pal", v); assert_eq!([(c.r * 255.0).round() as u8, (c.g * 255.0).round() as u8, (c.b * 255.0).round() as u8], erwartet, "Atari-Farbwert ${v:02X}");
        }
        assert_eq!(farben.eintraege("atari8", "pal").len(), 128, "alle 128 GTIA-Farben wählbar"); assert_eq!(farben.eintraege("atari8", "pal")[29].0, 58);
        // Spielerfarben der Datei kommen aus dem Modell, je Frame (Spieler) die eigene; Ziel der Datei ist Atari
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); einstellungen::test_konfig(std::env::temp_dir().join("vcgui-konf-atari"));
        assert!(app.projekt_oeffnen(repo.join("examples/atari/pmg_ship"))); app.oeffne("graphics/ship.vcpmg");
        assert_eq!(app.studios["graphics/ship.vcpmg"].doc.target, "atari8");
        for (i, w) in [58usize, 134, 196, 14].into_iter().enumerate() { let d = &mut app.studios.get_mut("graphics/ship.vcpmg").unwrap().doc; d.frame_waehlen(i); assert_eq!(d.farb_index(1), Some(w), "Spieler P{i}"); assert_eq!(d.farb_index(0), None); }
        // Zeichensatz-Studio (Atari): Ziel atari8, Vorder-/Hintergrund aus der Palette des Ziels
        let (mut app2, _) = App::neu(); app2.einst = Einstellungen::default(); assert!(app2.projekt_oeffnen(repo.join("examples/atari/gfx_demo"))); app2.oeffne("graphics/afont.vcchar");
        let dz = &app2.studios["graphics/afont.vcchar"].doc; assert_eq!(dz.target, "atari8"); let (fg, bg) = (dz.farb_index(1).unwrap(), dz.farb_index(0).unwrap());
        assert_eq!(farben.farbe("atari8", "pal", fg), farben.farbe("atari8", "pal", fg)); assert_ne!(farben.farbe("atari8", "pal", fg), farben.farbe("c64", "pal", fg), "Atari-Wert ${fg:02X} nicht als C64-Farbe gezeigt (sonst falscher Pfad)"); let _ = bg;
        assert_eq!(app.system_id(), "atari8", "Startziel zeigt das System der Datei, nicht C64");
    }
    #[test] fn studio_kopieren_einfuegen_alle_vier_wege_mit_realer_ereignisfolge() {
        use iced::{keyboard::{Event as K, Modifiers}, mouse, Event, Point};
        let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite"; let _ = simuliere(&mut app, &[]);
        let mitte = Point::new(740.0, 380.0);
        let tastendruck = |app: &mut App, kuerzel: &str| {
            // Cmd drücken, Taste drücken/loslassen, Cmd loslassen, Mausbewegung dazwischen: wie im Programm; Tasten erreichen `Taste` nur, wenn die Ansicht sie ignoriert
            let (ev, key, mods) = kuerzel_ev(kuerzel);
            for e in [Event::Keyboard(K::ModifiersChanged(Modifiers::COMMAND)), Event::Mouse(mouse::Event::CursorMoved { position: Point::new(mitte.x + 1.0, mitte.y) }), ev.clone()] {
                let (msgs, st) = ui_status(app, e.clone(), mitte); for m in msgs { let _ = app.update(m); }
                if let Event::Keyboard(K::KeyPressed { .. }) = e { if st.iter().all(|s| *s == iced::event::Status::Ignored) { let _ = app.update(Msg::Taste(key.clone(), mods)); } }
            }
            let Event::Keyboard(K::KeyPressed { key: k2, modified_key, physical_key, location, .. }) = ev else { unreachable!() };
            for e in [Event::Keyboard(K::KeyReleased { key: k2, modified_key, physical_key, location, modifiers: Modifiers::COMMAND }), Event::Keyboard(K::ModifiersChanged(Modifiers::empty()))] { let (msgs, _) = ui_status(app, e, mitte); for m in msgs { let _ = app.update(m); } }
        };
        let menue = |app: &mut App, id: &str| { let _ = app.update(Msg::Menue(id.into())); };
        let d = |app: &App| app.studios[rel].doc.frames[app.studios[rel].doc.fi].clone();
        let leer = |f: &Vec<Vec<u8>>| f.iter().flatten().all(|v| *v == 0);
        for (kopie, einfuegen) in [(true, true), (true, false), (false, true), (false, false)] {
            for ausschneiden in [false, true] {
                let (mut app, tmp2) = studio_app(); let _ = simuliere(&mut app, &[]);
                let orig = d(&app);
                // Auswahl der linken oberen Ecke 4×3 per Auswahlwerkzeug aufziehen (S, dann Ziehen mit der Maus ist in anderen Tests geprüft: hier direkt der Zustand)
                app.studios.get_mut(rel).unwrap().doc.sel = Some((2, 3, 5, 5)); let ausschnitt: Vec<Vec<u8>> = (3..=5).map(|y| orig[y][2..=5].to_vec()).collect();
                if kopie { tastendruck(&mut app, if ausschneiden { "Cmd+X" } else { "Cmd+C" }) } else { menue(&mut app, if ausschneiden { "ausschneiden" } else { "kopieren" }) }
                assert_eq!(app.studios[rel].doc.clip.as_ref(), Some(&ausschnitt), "Zwischenablage nach {} (Tastatur: {kopie})", if ausschneiden { "Ausschneiden" } else { "Kopieren" });
                assert!(app.studios[rel].doc.status.starts_with(if ausschneiden { "Ausgeschnitten (4×3" } else { "Kopiert (4×3" }), "{}", app.studios[rel].doc.status);
                if ausschneiden { assert!((3..=5).all(|y| d(&app)[y][2..=5].iter().all(|v| *v == 0)), "ausgeschnitten") } else { assert_eq!(d(&app), orig) }
                // Einfügen in ein anderes Frame (links oben der Auswahl wird auf (0,0) gesetzt), per Tastatur oder Menü
                let (n_frames, _) = (app.studios[rel].doc.frames.len(), 0); assert!(n_frames >= 2);
                app.studios.get_mut(rel).unwrap().doc.frame_waehlen(1); app.studios.get_mut(rel).unwrap().doc.sel = Some((0, 0, 0, 0)); let leer_vor = { let mut f = d(&app); for y in 0..3 { for x in 0..4 { f[y][x] = 0 } } f };
                app.studios.get_mut(rel).unwrap().doc.setze(0, 0, 0);
                if einfuegen { tastendruck(&mut app, "Cmd+V") } else { menue(&mut app, "einfuegen") }
                let f = d(&app); for y in 0..3 { for x in 0..4 { assert_eq!(f[y][x], ausschnitt[y][x], "Pixel {x},{y} eingefügt (Tastatur: {kopie}/{einfuegen}, ausschneiden: {ausschneiden})") } }
                assert!(app.studios[rel].doc.status.starts_with("Eingefügt (4×3 Pixel) bei 0,0"), "{}", app.studios[rel].doc.status); let _ = (leer_vor, leer(&f));
                let _ = std::fs::remove_dir_all(&tmp2);
            }
        }
        // ohne Auswahl: Einfügen an der Zelle unter der Maus (echte Mausbewegung über der Fläche meldet die Zelle), sonst links oben
        { let (mut app, tmp3) = studio_app(); let _ = simuliere(&mut app, &[]); app.studios.get_mut(rel).unwrap().doc.sel = Some((2, 3, 3, 4)); let _ = app.befehl(Cmd::Kopieren); app.studios.get_mut(rel).unwrap().doc.sel = None;
          let _ = app.befehl(Cmd::Einfuegen); assert!(app.studios[rel].doc.status.ends_with("bei 0,0"), "{}", app.studios[rel].doc.status);
          app.studios.get_mut(rel).unwrap().doc.sel = None; let q = zelle_punkt(&mut app, rel, 420.0, 900.0, 200.0, 600.0); let p = Point::new(q.x + 8.0, q.y); let msgs = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: q }), q), (Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]);
          let z = app.studios[rel].ueber.unwrap_or_else(|| panic!("Zelle unter der Maus: {msgs:?}")); let _ = app.befehl(Cmd::Einfuegen); assert!(app.studios[rel].doc.status.ends_with(&format!("bei {},{}", z.0, z.1)), "{}", app.studios[rel].doc.status);
          let _ = std::fs::remove_dir_all(&tmp3); }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn frame_vorschau_behaelt_eigene_farbe_unabhaengig_vom_aktiven_frame() {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."); einstellungen::test_konfig(std::env::temp_dir().join("vcgui-konf-vorschau"));
        // Player/Missile: je Frame (Spieler) die eigene Farbe, egal welches Frame gewählt ist
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); assert!(app.projekt_oeffnen(repo.join("examples/atari/pmg_ship"))); app.oeffne("graphics/ship.vcpmg");
        let erwartet = [58usize, 134, 196, 14];
        for aktiv in 0..4 { let d = &mut app.studios.get_mut("graphics/ship.vcpmg").unwrap().doc; d.frame_waehlen(aktiv); for (i, w) in erwartet.iter().enumerate() { assert_eq!(d.farb_index_frame(i, 1), Some(*w), "Vorschau Frame {i}, aktiv {aktiv}"); assert_eq!(d.farb_index_frame(i, 0), None) } }
        // Die gezeichneten Vorschau- und Onion-Skin-Farben im Bild: Frame P1 sieht bei aktivem P0 und P3 gleich aus (Bildvergleich der Vorschau-Pixel)
        let farben = studio::Farben::laden(&app.data); let sd = &app.studios["graphics/ship.vcpmg"].doc; assert_ne!(farben.farbe(&sd.target, "pal", erwartet[0]), farben.farbe(&sd.target, "pal", erwartet[1]));
        // C64-Sprite mit zwei Modi: Hires-Frame und Multicolor-Frame behalten ihre Farbrollen unabhängig vom gewählten Frame
        let (mut app2, tmp) = studio_app(); let rel = "hero.vcsprite"; let _ = app2.update(Msg::StudioAkt(rel.into(), studio::Aktion::FrameDup));
        { let d = &mut app2.studios.get_mut(rel).unwrap().doc; d.frame_waehlen(1); d.modus_wechseln(false); d.frame_waehlen(0); let mc0 = d.ist_multicolor(0); let mc1 = d.ist_multicolor(1); assert_ne!(mc0, mc1, "zwei Modi");
          let (m, h) = if mc0 { (0, 1) } else { (1, 0) };
          for aktiv in [0, 1] { d.frame_waehlen(aktiv); assert_eq!(d.farb_index_frame(m, 1), Some(d.farbe("mc1", 2)), "Multicolor-Frame zeigt Multicolor 1 (aktiv {aktiv})"); assert_eq!(d.farb_index_frame(m, 2), Some(d.farbe("sprite", 1))); assert_eq!(d.farb_index_frame(m, 3), Some(d.farbe("mc2", 6))); assert_eq!(d.farb_index_frame(h, 1), Some(d.farbe("sprite", 1)), "Hires-Frame zeigt Sprite-Farbe (aktiv {aktiv})"); } }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn studio_auswahl_zwischenablage_kontextmenue_png_und_vorlage() {
        use iced::{mouse, Event, Point}; use studio::Aktion as A; let (mut app, tmp) = studio_app(); let rel = "hero.vcsprite";
        let akt = |app: &mut App, a: A| { let _ = app.update(Msg::StudioAkt(rel.into(), a)); };
        let _ = simuliere(&mut app, &[]);
        // Auswahl: alles, Kopieren/Ausschneiden/Einfügen über die Befehlstabelle (Cmd+C/X/V), Entf löscht die Auswahl, Esc hebt sie auf
        let _ = app.update(Msg::Taste(iced::keyboard::Key::Character("a".into()), iced::keyboard::Modifiers::COMMAND)); let (w, h) = { let d = &app.studios[rel].doc; (d.width, d.height) }; assert_eq!(app.studios[rel].doc.sel, Some((0, 0, w - 1, h - 1)));
        let vorher = app.studios[rel].doc.frames[0].clone(); let _ = app.befehl(Cmd::Kopieren); assert!(app.studios[rel].doc.clip.is_some() && app.studios[rel].doc.frames[0] == vorher, "Kopieren ändert nichts");
        let _ = app.befehl(Cmd::Ausschneiden); assert!(app.studios[rel].doc.frames[0].iter().all(|z| z.iter().all(|v| *v == 0)), "Ausschneiden leert die Auswahl");
        let _ = app.befehl(Cmd::Einfuegen); assert_eq!(app.studios[rel].doc.frames[0], vorher, "Einfügen stellt den Inhalt wieder her");
        let _ = app.update(Msg::Taste(iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete), iced::keyboard::Modifiers::empty())); assert!(app.studios[rel].doc.frames[0].iter().all(|z| z.iter().all(|v| *v == 0)), "Entf löscht die Auswahl");
        let _ = app.update(Msg::Taste(iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape), iced::keyboard::Modifiers::empty())); assert!(app.studios[rel].doc.sel.is_none(), "Esc hebt die Auswahl auf");
        let _ = app.befehl(Cmd::Rueckgaengig); let _ = app.befehl(Cmd::Rueckgaengig);
        // Kontextmenü: Rechtsklick mit dem Auswahlwerkzeug; Einträge werden nach Zustand aktiv
        akt(&mut app, A::Werkzeug(vc_ide_model::pixel::Tool::Select)); let p = Point::new(730.0, 380.0);
        let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)), p)]);
        let k = app.kontext.as_ref().expect("Menü der Zeichenfläche"); assert_eq!(k.art, Art::Flaeche); let (z, c) = app.kontext_zeilen(k); assert!(c.contains(&Some(Cmd::Exportieren)) && z.iter().any(|x| x.name.starts_with("Einfügen")), "{:?}", z.iter().map(|x| x.name.clone()).collect::<Vec<_>>());
        let _ = app.update(Msg::KontextZu); let _ = simuliere(&mut app, &[]);
        // PNG-Import (ohne Dateidialog): Frame ändert sich, Rückgängig stellt her
        let (bw, bh) = (24usize, 21usize); let rgba: Vec<u8> = (0..bw * bh).flat_map(|i| if i % bw < 8 && i / bw < 5 { [0x88, 0x39, 0x32, 255] } else { [0, 0, 0, 0] }).collect();
        std::fs::write(tmp.join("bild.png"), vc_media::png::encode_rgba(bw, bh, &rgba)).unwrap(); let f0 = app.studios[rel].doc.frames[0].clone(); app.png_laden(rel, &tmp.join("bild.png"));
        assert!(app.log.last().unwrap().contains("PNG importiert"), "{:?}", app.log.last()); assert!(app.studios[rel].doc.frames[0] != f0); akt(&mut app, A::Undo); assert_eq!(app.studios[rel].doc.frames[0], f0);
        // Zeichensatz-Vorlage (Verweis auf ROM-Zeichensatz; fehlt das ROM, ist der Knopf ausgegraut, nie ein Absturz)
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."); std::fs::copy(repo.join("examples/atari/gfx_demo/graphics/afont.vcchar"), tmp.join("afont.vcchar")).unwrap(); app.oeffne("afont.vcchar");
        let _ = simuliere(&mut app, &[]); let vl = app.studios["afont.vcchar"].doc.vorlagen(&app.data); if let Some((r_, _, fehlt)) = vl.first() { if !fehlt { let _ = app.update(Msg::StudioAkt("afont.vcchar".into(), A::Vorlage(r_.clone()))); assert_eq!(app.studios["afont.vcchar"].doc.vorlage_aktuell(), *r_) } }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_sprite_studio() { let (mut app, tmp) = studio_app(); foto(&mut app, &[], "/tmp/vc-studio.png"); foto_gross(&mut app, &[], "/tmp/vc-studio-900.png", 900, 700); foto_gross(&mut app, &[], "/tmp/vc-studio-720.png", 720, 800); let _ = std::fs::remove_dir_all(&tmp); }
    #[test] fn debug_flaeche_in_der_schublade_mit_trennlinie_und_schmalem_rueckfall() {
        let (mut app, tmp) = debug_app(); let _ = app.update(Msg::DebugTick); app.schublade = 3; app.layout.aufklappen(Bereich::Schublade);
        assert!(!app.layout.offen(Bereich::Panel), "das rechte Panel bleibt, wie es ist"); assert!((app.einst.debug_hoehe - 0.38).abs() < 1e-6, "Vorgabe ≈ 38 %");
        let _ = simuliere(&mut app, &[]);
        // Trennlinie ziehen: Höhe folgt der Maus (begrenzt auf 20–70 %) und wird beim Loslassen gemerkt
        let konf = tmp.join("konf"); crate::einstellungen::test_konfig(&konf);
        let _ = app.update(Msg::TrennerMaus(iced::Point::new(600.0, 300.0))); assert!((app.einst.debug_hoehe - 0.38).abs() < 1e-6, "ohne Start bewegt sich nichts");
        let _ = app.update(Msg::TrennerStart); let _ = app.update(Msg::TrennerMaus(iced::Point::new(600.0, 400.0))); let h = app.einst.debug_hoehe; assert!(h > 0.4 && h < 0.7, "{h}");
        let _ = app.update(Msg::TrennerMaus(iced::Point::new(600.0, 5.0))); assert!((app.einst.debug_hoehe - 0.7).abs() < 1e-6, "obere Grenze"); let _ = app.update(Msg::TrennerMaus(iced::Point::new(600.0, 790.0))); assert!((app.einst.debug_hoehe - 0.2).abs() < 1e-6, "untere Grenze");
        let _ = app.update(Msg::TrennerMaus(iced::Point::new(600.0, 400.0))); let gemerkt = app.einst.debug_hoehe; let _ = app.update(Msg::TrennerLos); assert!(!app.trenner && Einstellungen::laden().debug_hoehe == gemerkt, "Wert gemerkt");
        let _ = simuliere(&mut app, &[]);
        // schmales Fenster (< 900): einspaltig mit Reitern Register · Disassembler · Speicher · Beobachten
        app.fenster = iced::Size::new(800.0, 700.0); assert!(app.schmal());
        for i in 0..4 { let _ = app.update(Msg::DebugSpalte(i)); assert_eq!(app.debug_spalte, i); let _ = simuliere(&mut app, &[]); }
        // Debug-Reiter in der Schublade endet mit der Sitzung: zurück auf Protokoll
        let _ = app.befehl(Cmd::DebugEnde); assert!(app.dbg.is_none()); assert_ne!(app.schublade, 3, "Reiter „Debug“ verschwindet mit der Sitzung");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn emulatorbild_wird_uebernommen_und_gezeichnet() {
        let (mut app, tmp) = debug_app();
        let mut snap = app.dbg.as_ref().unwrap().snapshot(); snap.can_frame = true; snap.can_text = true; snap.frame_sources = vec!["VIC-II".into(), "VDC".into()];
        let (w, h) = (384usize, 272usize); let rgba: Vec<u8> = (0..w * h).flat_map(|i| [(i % 256) as u8, ((i / w) % 256) as u8, 128, 255]).collect();
        snap.frame = Some(std::sync::Arc::new(vc_emu::streams::video::Frame { width: w, height: h, rgba, frame_no: 1 })); snap.frame_seq = 1;
        app.dbg = Some(debug::Debugger::demo(snap));
        let _ = app.update(Msg::DebugTick); let b = app.emu_bild.as_ref().expect("Bild übernommen"); assert_eq!((b.0, b.2, b.3), (1, 384, 272));
        app.panel_seite = PanelSeite::Emulator; app.layout.aufklappen(Bereich::Panel);
        for m in [Msg::EmuAuto(false), Msg::EmuLive(true), Msg::EmuBild, Msg::EmuQuelle(1), Msg::EmuText("run".into()), Msg::EmuSenden] { let _ = app.update(m); }
        let _ = simuliere(&mut app, &[]); assert!(app.emu_text.is_empty());
        let _ = app.befehl(Cmd::DebugEnde); assert!(app.emu_bild.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Messung (Konzept 14.10): Bildaufbau und Zeichnen eines 384×272-Bildes mit dem CPU-Renderer, 50 Bilder
    /// Mit echtem VICE (nur von Hand: `cargo test -p vc-gui echter_vice -- --ignored --nocapture`)
    #[test] #[ignore] fn echter_vice_speicher_register_disassembler() {
        let data = vc_ide::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let tmp = std::env::temp_dir().join(format!("vcgui-vice-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("main.asm"), ".target c64\n.org $0801\nstart: lda #$00\n  sta $d020\n  inc $d020\n  jmp start\n").unwrap();
        let b = vc_ide::build::build(&vc_ide::build::Ctx { root: &tmp, data: &data }, "main.asm", None).unwrap(); assert_eq!(b["ok"], true, "{b}");
        let cpu = vc_target::load_target(&data, "c64").and_then(|t| vc_target::load_cpu(&data, &t.cpu)).unwrap();
        let spec = vc_ide::debug::LaunchSpec { artifact: tmp.join(b["artifact"].as_str().unwrap()).to_string_lossy().into_owned(), target: "c64".into(), adapter: "emulator".into(), host: None, password: None, entry: None, extra_args: vec![] };
        let d = debug::Debugger::start(spec, cpu, HashMap::new(), vec![]);
        let warte = |d: &debug::Debugger, f: &dyn Fn(&debug::Snap) -> bool, what: &str| { let t0 = std::time::Instant::now(); loop { let s = d.snapshot(); if f(&s) { return s } assert!(t0.elapsed().as_secs() < 40, "Zeit: {what}: error={:?}", s.error); std::thread::sleep(std::time::Duration::from_millis(100)) } };
        warte(&d, &|s| !s.starting, "start");
        std::thread::sleep(std::time::Duration::from_secs(2));
        d.send(debug::Cmd::Pause); let s = warte(&d, &|s| !s.running && s.have_regs, "pause"); eprintln!("angehalten PC {:04X} Fehler {:?}", s.pc, s.error);
        d.send(debug::Cmd::Mem(0xD020)); let s = warte(&d, &|s| s.mem_addr == 0xD020 || s.error.is_some(), "mem"); eprintln!("Speicher D020: addr {:04X} len {} Fehler {:?}", s.mem_addr, s.mem.len(), s.error);
        d.send(debug::Cmd::Mem(0x0801)); let s = warte(&d, &|s| s.mem_addr == 0x0801 || s.error_seq > 0, "mem2"); eprintln!("Speicher 0801: len {} Fehler {:?} erste {:?}", s.mem.len(), s.error, &s.mem[..8.min(s.mem.len())]);
        d.send(debug::Cmd::Watches(vec!["$d020".into(), "pc".into(), "d020".into()])); d.send(debug::Cmd::Mem(0x0400)); let s = warte(&d, &|s| s.watch.len() == 3, "beobachten"); eprintln!("Beobachten: {:?}", s.watch);
        d.send(debug::Cmd::Step("over")); std::thread::sleep(std::time::Duration::from_millis(500)); let s = d.snapshot(); eprintln!("nach Schritt PC {:04X} Fehler {:?}", s.pc, s.error);
        drop(d); std::thread::sleep(std::time::Duration::from_secs(2)); let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Attrappe eines Emulators, dessen Verbindung abbricht (wie ein beendetes VICE: Schreiben auf die geschlossene Verbindung → „broken pipe“)
    struct Abbruch { tot: std::sync::Arc<std::sync::atomic::AtomicBool> }
    impl Abbruch { fn pruefe(&self) -> Result<(), String> { if self.tot.load(std::sync::atomic::Ordering::SeqCst) { Err("broken pipe (os error 32)".into()) } else { Ok(()) } } }
    impl vc_emu::Backend for Abbruch {
        fn name(&self) -> &str { "Attrappe" }
        fn state(&mut self) -> vc_emu::RunState { vc_emu::RunState::Stopped }
        fn registers(&mut self) -> Result<vc_emu::Regs, String> { self.pruefe()?; Ok(vc_emu::Regs { pc: 0x0801, a: 0, x: 0, y: 0, sp: 0xFD, flags: 0x24, extra: vec![] }) }
        fn read_mem(&mut self, _a: u16, n: usize) -> Result<Vec<u8>, String> { self.pruefe()?; Ok(vec![0; n]) }
        fn write_mem(&mut self, _a: u16, _d: &[u8]) -> Result<(), String> { self.pruefe() }
        fn step(&mut self) -> Result<(), String> { self.pruefe() }
        fn cont(&mut self) -> Result<(), String> { self.pruefe() }
        fn can_pause(&self) -> bool { true }
        fn pause(&mut self) -> Result<(), String> { self.pruefe() }
        fn set_breakpoint(&mut self, _a: u16) -> Result<(), String> { self.pruefe() }
        fn clear_breakpoint(&mut self, _a: u16) -> Result<(), String> { self.pruefe() }
        fn breakpoints(&self) -> Vec<u16> { vec![] }
        fn stop(&mut self) {}
    }
    #[test] fn verbindungsabbruch_zum_emulator_wird_verstaendlich_gemeldet_und_beendet_die_sitzung() {
        let data = vc_ide::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let cpu = vc_target::load_target(&data, "c64").and_then(|t| vc_target::load_cpu(&data, &t.cpu)).unwrap();
        let tot = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let d = debug::Debugger::start_fake(Box::new(Abbruch { tot: tot.clone() }), cpu);
        let warte = |d: &debug::Debugger, f: &dyn Fn(&debug::Snap) -> bool| { let t0 = std::time::Instant::now(); loop { let s = d.snapshot(); if f(&s) { return s } assert!(t0.elapsed().as_secs() < 10, "Zeit: {:?} {:?}", s.error, s.error_raw); std::thread::sleep(std::time::Duration::from_millis(50)) } };
        let s = warte(&d, &|s| !s.starting && s.have_regs); assert!(s.error.is_none() && !s.ended);
        // Speicherabfrage bei lebender Verbindung klappt
        d.send(debug::Cmd::Mem(0xD020)); let s = warte(&d, &|s| s.mem_addr == 0xD020); assert_eq!(s.mem.len(), 256);
        // Beobachten: `$d020` ist eine Adresse (Byte), `a` ein Register, `d020` ohne $ wird nicht geraten
        d.send(debug::Cmd::Watches(vec!["$d020".into(), "0xd020".into(), "53280".into(), "pc".into(), "[$d020] + 1".into(), "d020".into()]));
        let s = warte(&d, &|s| s.watch.len() == 6); let w = &s.watch;
        assert_eq!((w[0].1.clone(), w[1].1.clone(), w[2].1.clone(), w[3].1.clone(), w[4].1.clone()), (Ok(0), Ok(0), Ok(0), Ok(0x0801), Ok(1)), "{w:?}");
        let f = w[5].1.clone().unwrap_err(); assert!(f.contains("Meinst du $d020?") && !f.starts_with("'d020'"), "{f}");
        // Verbindung bricht ab, danach Speicherabfrage und Schritt
        tot.store(true, std::sync::atomic::Ordering::SeqCst); d.send(debug::Cmd::Mem(0xD020)); d.send(debug::Cmd::Step("over"));
        let s = warte(&d, &|s| s.ended);
        let e = s.error.clone().expect("Fehlertext"); assert!(e.contains("Verbindung zum Emulator ist verloren") && !e.contains("os error") && !e.to_lowercase().contains("pipe"), "{e}");
        assert!(s.error_raw.as_deref().is_some_and(|r| r.contains("os error 32")), "roher Text bleibt für das Protokoll");
        // die App übernimmt das: Sitzung geordnet beendet, Panel zurück, Protokoll mit Text und Einzelheit
        let (mut app, tmp) = debug_app(); app.dbg = Some(d); app.panel_seite = PanelSeite::Debug;
        let _ = app.update(Msg::DebugTick);
        assert!(app.dbg.is_none() && app.panel_seite != PanelSeite::Debug, "Sitzung beendet");
        assert!(app.log.iter().any(|l| l.contains("Verbindung zum Emulator ist verloren") && !l.contains("os error")) && app.log.iter().any(|l| l.contains("Einzelheit") && l.contains("os error 32")) && app.log.iter().any(|l| l.contains("Debug-Sitzung beendet")), "{:?}", app.log);
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn messung_emulatorbild() {
        let (mut app, tmp) = debug_app(); app.dbg = None; app.layout.aufklappen(Bereich::Panel); app.panel_seite = PanelSeite::Emulator;
        let (w, h) = (384usize, 272usize);
        let t0 = std::time::Instant::now(); let mut handles = vec![];
        for n in 0..50u32 { let rgba: Vec<u8> = (0..w * h).flat_map(|i| [(i as u32 + n) as u8, 64, 128, 255]).collect(); handles.push(image::Handle::from_rgba(w as u32, h as u32, rgba)) }
        let aufbau = t0.elapsed().as_secs_f64() * 1000.0 / 50.0;
        let t1 = std::time::Instant::now();
        for hd in handles { app.emu_bild = Some((1, hd, w, h)); let _ = simuliere(&mut app, &[]); }
        let zeichnen = t1.elapsed().as_secs_f64() * 1000.0 / 50.0;
        eprintln!("MESSUNG Emulatorbild 384x272: Handle aus RGBA {aufbau:.2} ms/Bild, Oberfläche bauen+zeichnen (tiny-skia, CPU, ganze Fenstersicht) {zeichnen:.2} ms/Bild");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// Tastendruck als Ereignis (mit Text, wie ihn ein echtes Fenster liefert)
    fn taste_ev(c: &str) -> iced::Event { use iced::keyboard::{key, Event as K, Key, Location, Modifiers}; iced::Event::Keyboard(K::KeyPressed { key: Key::Character(c.into()), modified_key: Key::Character(c.into()), physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified), location: Location::Standard, modifiers: Modifiers::empty(), text: Some(c.into()), repeat: false }) }
    /// Taste mit Modifikatoren aus einem Kürzel der Befehlstabelle („Cmd+Umschalt+Enter“, „F10“, „Cmd+Komma“) als Ereignis
    fn kuerzel_ev(tasten: &str) -> (iced::Event, iced::keyboard::Key, iced::keyboard::Modifiers) {
        use iced::keyboard::{key, key::Named, Event as K, Key, Location, Modifiers};
        let (cmd, sh, alt, taste) = befehle::zerlege(tasten).unwrap();
        let mut mods = Modifiers::empty(); if cmd { mods |= Modifiers::COMMAND } if sh { mods |= Modifiers::SHIFT } if alt { mods |= Modifiers::ALT }
        let k = match taste { "Enter" => Key::Named(Named::Enter), "Links" => Key::Named(Named::ArrowLeft), "Rechts" => Key::Named(Named::ArrowRight), "Plus" => Key::Character("+".into()), "Minus" => Key::Character("-".into()), "Komma" => Key::Character(",".into()),
            f if f.starts_with('F') && f.len() > 1 => Key::Named(match f { "F1" => Named::F1, "F2" => Named::F2, "F5" => Named::F5, "F6" => Named::F6, "F7" => Named::F7, "F8" => Named::F8, "F9" => Named::F9, "F10" => Named::F10, "F11" => Named::F11, "F12" => Named::F12, _ => panic!("Taste {f}") }),
            c => Key::Character(c.to_lowercase().as_str().into()) };
        (iced::Event::Keyboard(K::KeyPressed { key: k.clone(), modified_key: k.clone(), physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified), location: Location::Standard, modifiers: mods, text: None, repeat: false }), k, mods)
    }
    /// Ereignis an die gebaute Ansicht geben, ohne die Nachrichten anzuwenden: liefert Nachrichten und den Status je Ereignis (erfasst oder ignoriert)
    fn ui_status(app: &mut App, e: iced::Event, p: iced::Point) -> (Vec<Msg>, Vec<iced::event::Status>) {
        use iced_runtime::user_interface::UserInterface;
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new({ schriften_laden(); vc_gui_ui::fonts::UI }, iced::Pixels(16.0)));
        let grenze = iced::Size::new(app.fenster.width / app.einst.zoom, app.fenster.height / app.einst.zoom); let mut msgs: Vec<Msg> = vec![];
        let cache = CACHE.with(|c| c.borrow_mut().take()).unwrap_or_default(); let (st, cache) = { let view = app.view(); let mut ui = UserInterface::build(view, grenze, cache, &mut renderer); let (_, st) = ui.update(&[e], iced::mouse::Cursor::Available(p), &mut renderer, &mut iced::advanced::clipboard::Null, &mut msgs); (st, ui.into_cache()) };
        CACHE.with(|c| *c.borrow_mut() = Some(cache)); (msgs, st)
    }
    #[test] fn alle_kuerzel_der_befehlstabelle_kommen_im_fokussierten_editor_durch() {
        use iced::{mouse, Event, Point};
        let tmp = std::env::temp_dir().join(format!("vcgui-kuerzel-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap(); std::fs::write(tmp.join("a.asm"), "nop\nlda #1\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.asm"); let p = Point::new(600.0, 120.0);
        let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]);
        let _ = simuliere(&mut app, &[(taste_ev("x"), p)]); assert!(app.editoren["a.asm"].text().contains('x'), "Editor hat den Fokus");
        // editoreigene Bearbeitungen: bleiben beim Editor (gleiche Wirkung wie die Tabelle sie vorsieht)
        let eigene = ["Cmd+Z", "Cmd+Umschalt+Z"];
        let vorher = app.editoren["a.asm"].text(); let mut geprueft = 0;
        for i in befehle::BEFEHLE { if i.tasten.is_empty() { continue }
            let (ev, k, mods) = kuerzel_ev(i.tasten);
            assert!(trifft(i.tasten, &k, mods), "Kürzel {} ({:?}) wird von der Tastenauswertung erkannt", i.tasten, i.cmd);
            if eigene.contains(&i.tasten) { continue }
            let (msgs, st) = ui_status(&mut app, ev, p);
            assert!(st.iter().all(|s| *s == iced::event::Status::Ignored), "Kürzel {} ({:?}) darf der Editor nicht verbrauchen: {st:?}", i.tasten, i.cmd);
            assert!(!msgs.iter().any(|m| matches!(m, Msg::Ed(_) | Msg::EdIn(..))), "Kürzel {} löst keine Editor-Eingabe aus: {msgs:?}", i.tasten);
            assert_eq!(app.editoren["a.asm"].text(), vorher, "Kürzel {} ändert den Text nicht", i.tasten); geprueft += 1;
        }
        assert!(geprueft >= 35, "{geprueft} Kürzel geprüft");
        // Cmd+Umschalt+Enter wirkt auf das Dokumentenfeld: maximiert, nochmal stellt wieder her
        app.fokus_bereich = Bereich::Dokument; let (_, k, mods) = kuerzel_ev("Cmd+Umschalt+Enter");
        let _ = app.update(Msg::Taste(k.clone(), mods)); assert_eq!(app.layout.anzeige(Bereich::Dokument), Anzeige::Maximiert, "maximiert");
        let _ = app.update(Msg::Taste(k, mods)); assert_eq!(app.layout.anzeige(Bereich::Dokument), Anzeige::Normal, "wiederhergestellt");
        // Enter ohne Modifikator schreibt weiter im Editor
        let (msgs, st) = ui_status(&mut app, iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter), modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter), physical_key: iced::keyboard::key::Physical::Unidentified(iced::keyboard::key::NativeCode::Unidentified), location: iced::keyboard::Location::Standard, modifiers: iced::keyboard::Modifiers::empty(), text: None, repeat: false }), p);
        assert!(st.contains(&iced::event::Status::Captured) && msgs.iter().any(|m| matches!(m, Msg::EdIn(..))), "einfaches Enter ist Editor-Sache");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn genau_ein_fokusbesitzer_suchfeld_palette_gehezu_und_dialog_schreiben_nie_in_den_code() {
        use iced::{mouse, Event, Point};
        let tmp = std::env::temp_dir().join(format!("vcgui-fokus-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.asm"), "nop\n").unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false; einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("a.asm");
        let code = |app: &App| app.editoren["a.asm"].text();
        let tippe = |app: &mut App, z: &str| { let p = Point::new(600.0, 300.0); let mut evs: Vec<(Event, Point)> = vec![(Event::Window(iced::window::Event::RedrawRequested(std::time::Instant::now())), p)]; evs.extend(z.chars().map(|c| (taste_ev(&c.to_string()), p))); let _ = simuliere(app, &evs); };
        // Klick in den Editor: Fokus dort, Tippen ändert den Code
        let p = Point::new(600.0, 120.0); let _ = simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), p), (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), p)]);
        tippe(&mut app, "x"); assert!(code(&app).contains('x'), "Editor hat den Fokus: {:?}", code(&app)); let basis = code(&app);
        // Cmd+F (und Ersetzen, Gehe zu, Palette): der Editor verliert die Tastatur, der Code bleibt unverändert
        for c in [Cmd::Suchen, Cmd::Ersetzen, Cmd::GeheZuZeile, Cmd::Palette] {
            let _ = app.befehl(c); assert!(!app.ed_aktiv, "{c:?}: Editor ist nicht mehr Fokusbesitzer");
            tippe(&mut app, "qwe"); assert_eq!(code(&app), basis, "{c:?}: Tippen darf den Code nicht ändern");
            let _ = app.update(Msg::Taste(iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape), iced::keyboard::Modifiers::empty())); assert!(app.ed_aktiv, "Esc gibt den Fokus an den Editor zurück");
            tippe(&mut app, "y"); assert!(code(&app).contains('y'), "{c:?}: nach Esc schreibt der Editor wieder"); let basis2 = code(&app); assert!(basis2.len() > basis.len());
            { let e = app.editoren.get_mut("a.asm").unwrap(); e.lines = basis.split('\n').map(String::from).collect(); e.cursor = Pos { line: 0, col: 0 }; e.version += 1; } let _ = simuliere(&mut app, &[]);
        }
        // Dialog „Neue Datei“ und Debug-Felder
        let _ = app.update(Msg::BaumNeu(String::new(), NeuArt::Asm)); assert!(!app.ed_aktiv); tippe(&mut app, "zzz"); assert_eq!(code(&app), basis, "Dialog: nichts im Code");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn debug_sitzung_wird_von_tests_nie_stillschweigend_beendet() {
        let (mut app, tmp) = debug_app(); assert!(app.dbg.is_some());
        // „Alle ausführen“ während der Sitzung: Sitzung bleibt, Tests laufen parallel (eigener Emulator)
        let _ = app.befehl(Cmd::TestsAusfuehren); assert!(app.dbg.is_some(), "Sitzung lebt"); assert!(app.log.iter().any(|l| l.contains("parallel zur Debug-Sitzung")));
        let t0 = std::time::Instant::now(); while app.tests.laeuft() && t0.elapsed().as_secs() < 30 { std::thread::sleep(std::time::Duration::from_millis(50)); let _ = app.update(Msg::TestTick); }
        let _ = app.update(Msg::TestFehler); assert!(app.dbg.is_some(), "auch „Fehlgeschlagene“ beendet nichts");
        // „Diesen Test debuggen“ würde die Sitzung beenden: erst Rückfrage; Abbrechen lässt alles, wie es ist
        let f = tmp.join("x.vctest"); let _ = app.update(Msg::TestDebug(f.clone(), "t1".into()));
        assert_eq!(app.rueckfrage, Some(Rueckfrage::TestDebug(f.clone(), "t1".into()))); assert!(app.dbg.is_some(), "vor der Antwort bleibt die Sitzung");
        let _ = simuliere(&mut app, &[]); let _ = app.update(Msg::RueckfrageNein); assert!(app.rueckfrage.is_none() && app.dbg.is_some());
        // Esc schließt die Rückfrage ohne Folgen
        let _ = app.update(Msg::TestDebug(f.clone(), "t1".into())); let _ = app.update(Msg::Taste(iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape), iced::keyboard::Modifiers::empty())); assert!(app.rueckfrage.is_none() && app.dbg.is_some());
        // Ja beendet die Sitzung ausdrücklich
        let _ = app.update(Msg::TestDebug(f, "t1".into())); let _ = app.update(Msg::RueckfrageJa); assert!(app.dbg.is_none() && app.rueckfrage.is_none());
        // Gegenrichtung: „Debuggen“ während einer Sitzung fragt zuerst; Tests laufen dabei unberührt weiter
        let (mut app, tmp2) = debug_app(); let _ = app.befehl(Cmd::Debuggen); assert_eq!(app.rueckfrage, Some(Rueckfrage::DebugNeu)); assert!(app.dbg.is_some());
        let _ = app.update(Msg::RueckfrageNein); assert!(app.dbg.is_some() && !app.baut);
        let _ = std::fs::remove_dir_all(&tmp); let _ = std::fs::remove_dir_all(&tmp2);
    }
    #[test] fn include_kontext_hinweis_wird_der_richtigen_offenen_datei_zugeordnet() {
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); let tmp = std::env::temp_dir().join(format!("vcgui-inc-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("lib.asm"), "nop\n").unwrap(); std::fs::write(tmp.join("zu.asm"), "nop\n").unwrap();
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false;
        app.oeffne("lib.asm"); app.lsp_gesendet.insert("lib.asm".into(), 1);
        let d = |z: usize, sev: u8, t: &str| lsp::Diag { zeile: z, fehler: sev == 1, hinweis: sev >= 3, text: t.into() };
        // Diagnosen kommen auch für Dateien, die nicht das aktive Dokument sind: offene Datei wird gemerkt, geschlossene ignoriert
        app.lsp_antwort(lsp::Antwort::Diagnose { uri: lsp::uri(&tmp, "zu.asm"), diag: vec![d(0, 1, "egal")] });
        assert!(!app.diagnosen.contains_key("zu.asm"), "geschlossene Datei wird ignoriert");
        app.lsp_antwort(lsp::Antwort::Diagnose { uri: lsp::uri(&tmp, "lib.asm"), diag: vec![d(0, 3, "Symbol fehlt – vermutlich in der einbindenden Datei definiert")] });
        let e = &app.editoren["lib.asm"]; assert!(e.hints.contains_key(&0) && e.errors.is_empty() && e.warns.is_empty(), "Hinweis ist weder Fehler noch Warnung");
        // Hover auf der Zeile nennt „Hinweis“ und den Text
        let n = app.name(); assert_eq!(n.as_deref(), Some("lib.asm"));
        let dd = app.diagnosen["lib.asm"][0].clone(); assert!(dd.hinweis && !dd.fehler);
        // Dezente Markierung: das Bild unterscheidet sich vom unmarkierten, ohne Zeilenfarbe wie bei Fehlern
        app.lsp_antwort(lsp::Antwort::Diagnose { uri: lsp::uri(&tmp, "lib.asm"), diag: vec![] }); assert!(app.editoren["lib.asm"].hints.is_empty(), "leere Meldung räumt den Hinweis weg");
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn neu_steht_im_kontextmenue_des_baums_und_die_untere_knopfzeile_ist_weg() {
        use iced::{keyboard::{key::Named, Key, Modifiers}, mouse, Event, Point};
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); let tmp = std::env::temp_dir().join(format!("vcgui-neu-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("main.asm"), "nop\n").unwrap();
        app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); app.papierkorb = false; einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("main.asm");
        let taste = |app: &mut App, n: Named| { let _ = app.update(Msg::Taste(Key::Named(n), Modifiers::empty())); };
        // Rechtsklick auf die freie Fläche des Baums (weit unter den Dateien): Menü „Dateibaum“ mit Neue Datei, Neuer Ordner, Neue Notiz
        let p = Point::new(120.0, 600.0); simuliere(&mut app, &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p), (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)), p)]);
        let k = app.kontext.as_ref().expect("Menü auf der freien Fläche"); assert_eq!(k.art, Art::BaumFlaeche);
        let (z, c) = app.kontext_zeilen(k); let namen: Vec<_> = z.iter().filter(|x| !x.name.is_empty()).map(|x| x.name.as_str()).collect(); assert_eq!(namen, ["Neue Datei …", "Neuer Ordner …", "Neue Notiz …"]); assert!(c.iter().flatten().count() == 3);
        // Enter wählt „Neue Datei …“: der Dialog öffnet; Art Text, Name tippen, anlegen
        taste(&mut app, Named::Enter); assert!(matches!(app.baum.as_ref().map(|b| &b.art), Some(BaumArt::Neu { .. })), "Dialog Neue Datei");
        let _ = app.update(Msg::BaumArtWahl(NeuArt::Text)); let _ = app.update(Msg::BaumText("ideen".into())); let _ = app.update(Msg::BaumOk);
        assert!(tmp.join("ideen.txt").is_file() && app.baum.is_none(), "Endung ergänzt, Datei angelegt"); assert_eq!(app.name().as_deref(), Some("ideen.txt"), "neue Datei ist geöffnet");
        // Neue Notiz …: Art Notiz im Ordner notes/
        let _ = app.update(Msg::KontextAuf(Art::BaumFlaeche, Point::new(120.0, 600.0), Some(String::new()))); let _ = app.update(Msg::KontextWahl(Cmd::NeueNotizDialog));
        let _ = app.update(Msg::BaumText("heute".into())); let _ = app.update(Msg::BaumOk); assert!(tmp.join("notes/heute.md").is_file(), "Notiz in notes/");
        // Neuer Ordner …
        let _ = app.update(Msg::KontextWahl(Cmd::NeuerOrdner)); let _ = app.update(Msg::BaumText("lib/musik".into())); let _ = app.update(Msg::BaumOk); assert!(tmp.join("lib/musik").is_dir());
        // Studio-Datei (Sprite) über den Dialog: Profil des Projekts, Datei entsteht im Grafikordner
        let _ = app.update(Msg::KontextWahl(Cmd::NeueDatei)); let g = vc_ide_model::project::Group::Sprite;
        if !app.studio_profile(g).is_empty() { let _ = app.update(Msg::BaumArtWahl(NeuArt::Studio(g))); let _ = app.update(Msg::BaumText("ball".into())); let _ = app.update(Msg::BaumOk); assert!(app.baum.is_none(), "{:?}", app.baum.as_ref().map(|b| b.fehler.clone())); assert!(app.namen.iter().any(|n| n.ends_with("ball.sprite") || n.contains("ball")), "{:?}", app.namen); } else { let _ = app.update(Msg::BaumAbbruch); }
        // Fehler bleiben im Dialog: vorhandener Name
        let _ = app.update(Msg::KontextWahl(Cmd::NeueDatei)); let _ = app.update(Msg::BaumText("main.asm".into())); let _ = app.update(Msg::BaumOk); assert!(app.baum.as_ref().is_some_and(|b| !b.fehler.is_empty())); let _ = app.update(Msg::BaumAbbruch);
        let _ = app.view();
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] #[ignore] fn bild_neu_dialog() {
        let tmp = std::env::temp_dir().join(format!("vcgui-nd-{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("main.asm"), ".target c64\nstart: lda #$20\n").unwrap(); std::fs::write(tmp.join("projekt.vcprj"), r#"{"format":1,"project":{"name":"p"},"targets":[{"system":"c64"}]}"#).unwrap();
        let (mut app, _) = App::neu(); app.einst = Einstellungen::default(); app.layout = Layout::vorgabe(); app.root = tmp.clone(); app.projekt_neu_laden(); app.tabs.clear(); app.andere = None; app.editoren.clear(); app.lsp_gesendet.clear(); einstellungen::test_konfig(tmp.join("konf"));
        app.oeffne("main.asm");
        let _ = app.update(Msg::KontextAuf(Art::BaumFlaeche, iced::Point::new(120.0, 500.0), Some(String::new())));
        foto(&mut app, &[], "/tmp/vc-baum-menue.png");
        let _ = app.update(Msg::KontextWahl(Cmd::NeueDatei)); let g = vc_ide_model::project::Group::Sprite; let _ = app.update(Msg::BaumArtWahl(NeuArt::Studio(g)));
        foto(&mut app, &[], "/tmp/vc-neu-dialog.png"); let _ = std::fs::remove_dir_all(&tmp);
    }
}
