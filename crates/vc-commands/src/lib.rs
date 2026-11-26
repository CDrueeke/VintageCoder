// SPDX-License-Identifier: GPL-3.0-or-later
//! Herausgelöst aus `vc-gui-ui/src/befehle.rs` (E0, K2); die Iced-Kopie bleibt bis E13 unverändert.
//! Die **eine** Befehlstabelle: Menü, Befehlspalette und Kürzel entstehen alle hieraus (Konzept Abschnitt 7).
//! Kürzel stehen plattformneutral als „Cmd+Umschalt+S“; `fuer_anzeige` macht daraus ⌘⇧S (macOS) oder Strg+Umschalt+S.

pub mod dynamisch;
pub use dynamisch::{Befehl, Zeile};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cmd {
    ProjektOeffnen, NeuerReiter, NeueNotiz, DateiOeffnen, Speichern, AlleSpeichern, DateiSchliessen,
    Rueckgaengig, Wiederholen, Suchen, Ersetzen, GeheZuZeile, InProjektSuchen,
    Palette, ProjektBereich, StudioVergroessern, StudioVerkleinern, StudioEinpassen, StudioRaster, StudioZwiebel, TrackerNoten, TrackerInstrumente, TrackerAblauf, Seitenleiste, Schublade, Panel, Zen, MdUmschalten, BasicNachAsm, Zahlenwerkzeuge, ZoomPlus, ZoomMinus, ZoomNull, AlsInclude, AlleAuswaehlen, AllesFalten, AllesEntfalten, Vervollstaendigen, LesezeichenUmschalten, NaechstesLesezeichen, VorigesLesezeichen, Korrigieren, ZurKlammer, LeerzeichenEntfernen, ZeilenSortieren, KommentarUmschalten, ProblemSpringen, ProblemKopieren, ProblemeAlleKopieren, ProtokollKopieren, ProtokollLeeren, ZeileFormatieren, DateiFormatieren, EditorSchriftGroesser, EditorSchriftKleiner, DarstellungZuruecksetzen, HellDunkel, HoherKontrast, Umbruch, Einstellungen, Startseite, ProjektSchliessen, Zeilenende, Kodierung, Beenden, ProjektNeu, Assistent, Ueber, Feedback, UpdatePruefen, ProjektOrdnerOeffnen, ProjektKopieren,  ProjektEinstellungen, DiskOeffnen, TurboBereitstellen, TurboImDiskEditor, TurboPruefen,
    Bauen, Starten, Debuggen, BauenAktiv, StartenAktiv, DebuggenAktiv, MdFett, MdKursiv, MdCode, MdDurchgestrichen, MdLink, MdBild, MdUeberschriftTiefer, MdUeberschriftHoeher, MdAufzaehlung, MdNummerierung, MdAufgaben, MdZitat, MdAufgabeUmschalten, MdTabelleNeu, MdTabelleAusrichten, MdAlsMarkdown, HilfeImReiter, ArtText, ArtAsm, ArtBasic, HexVorschlaege, StdZeichensatz, ZeichensatzAusRom, ZeichensatzAusPng, AssistentRoms, AssistentEmulatoren, AssistentHardware, AssistentDesign, AssistentNeues, DebugWeiter, DebugPause, DebugUeber, DebugHinein, DebugRaus, DebugEnde, DebugHierher, TestsAusfuehren, Exportieren, PngLaden, NaechstesProblem, VorigesProblem,
    NaechsterReiter, VorigerReiter, NeueDatei, NeuerOrdner, NeueNotizDialog, ZielOeffnen, DateiUmbenennen, DateiLoeschen, PfadKopieren, ImOrdnerZeigen, HexAnsicht, BaumRueckgaengig, Ausschneiden, Kopieren, Einfuegen, GeheZuDefinition, HaltepunktUmschalten, ReiterAndereHaelfte, Teilen, ReiterLinks, ReiterRechts, MaximiereFokus, SpalteMax, DokumentMax, SchubladeMax, PanelMax, LayoutZuruecksetzen, Hilfe,
    BasicInLabels, BasicInNummern, BasicNeuNummerieren,
}

/// Wann ein Befehl nutzbar ist (sonst ausgegraut, mit Grund)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Braucht { Nichts, Projekt, Editor }

pub struct Info { pub cmd: Cmd, pub menue: &'static str, pub tasten: &'static str, pub braucht: Braucht, pub id: &'static str }
impl Info {
    /// Name des Befehls in der aktuellen Sprache (Katalog `cmd-<Kennung>`)
    pub fn name(&self) -> String { vc_i18n::translate(&format!("cmd-{}", self.id), &[]) }
}

const fn i(cmd: Cmd, id: &'static str, menue: &'static str, tasten: &'static str, braucht: Braucht) -> Info { Info { cmd, menue, tasten, braucht, id } }
use Braucht::*;

pub const BEFEHLE: &[Info] = &[
    i(Cmd::Ueber, "ueber", "Hilfe", "", Nichts),
    i(Cmd::Feedback, "feedback", "Hilfe", "", Nichts),
    i(Cmd::UpdatePruefen, "update_pruefen", "Hilfe", "", Nichts),
    i(Cmd::Assistent, "assistent", "Hilfe", "", Nichts),
    i(Cmd::MdFett, "md_fett", "Bearbeiten", "", Editor),
    i(Cmd::MdKursiv, "md_kursiv", "Bearbeiten", "Cmd+I", Editor),
    i(Cmd::MdCode, "md_code", "Bearbeiten", "Cmd+E", Editor),
    i(Cmd::MdDurchgestrichen, "md_durchgestrichen", "Bearbeiten", "Cmd+Umschalt+X", Editor),
    i(Cmd::MdLink, "md_link", "Bearbeiten", "Cmd+Umschalt+K", Editor),
    i(Cmd::MdBild, "md_bild", "Bearbeiten", "", Editor),
    i(Cmd::MdUeberschriftTiefer, "md_tiefer", "Bearbeiten", "", Editor),
    i(Cmd::MdUeberschriftHoeher, "md_hoeher", "Bearbeiten", "", Editor),
    i(Cmd::MdAufzaehlung, "md_aufzaehlung", "Bearbeiten", "", Editor),
    i(Cmd::MdNummerierung, "md_nummerierung", "Bearbeiten", "", Editor),
    i(Cmd::MdAufgaben, "md_aufgaben", "Bearbeiten", "", Editor),
    i(Cmd::MdZitat, "md_zitat", "Bearbeiten", "", Editor),
    i(Cmd::MdAufgabeUmschalten, "md_aufgabe", "Bearbeiten", "", Editor),
    i(Cmd::MdTabelleNeu, "md_tabelle", "Bearbeiten", "", Editor),
    i(Cmd::MdTabelleAusrichten, "md_ausrichten", "Bearbeiten", "", Editor),
    i(Cmd::MdAlsMarkdown, "md_als", "Bearbeiten", "", Editor),
    i(Cmd::HilfeImReiter, "hilfe_reiter", "Hilfe", "", Nichts),
    i(Cmd::ArtText, "art_text", "Bearbeiten", "", Editor),
    i(Cmd::ArtAsm, "art_asm", "Bearbeiten", "", Editor),
    i(Cmd::ArtBasic, "art_basic", "Bearbeiten", "", Editor),
    i(Cmd::HexVorschlaege, "hex_vorschlaege", "Bearbeiten", "", Editor),
    i(Cmd::StdZeichensatz, "std_zeichensatz", "Datei", "", Projekt),
    i(Cmd::ZeichensatzAusRom, "zeichensatz_rom", "Datei", "", Projekt),
    i(Cmd::ZeichensatzAusPng, "zeichensatz_png", "Datei", "", Projekt),
    i(Cmd::AssistentNeues, "assistent_neues", "Hilfe", "", Nichts),
    i(Cmd::AssistentRoms, "assistent_roms", "Hilfe", "", Nichts),
    i(Cmd::AssistentEmulatoren, "assistent_emulatoren", "Hilfe", "", Nichts),
    i(Cmd::AssistentHardware, "assistent_hardware", "Hilfe", "", Nichts),
    i(Cmd::AssistentDesign, "assistent_design", "Hilfe", "", Nichts),
    i(Cmd::ProjektNeu, "projekt_neu", "Datei", "Cmd+Alt+N", Nichts),
    i(Cmd::ProjektEinstellungen, "projekt_einstellungen", "Projekt", "", Projekt),
    i(Cmd::TurboBereitstellen, "turbo_bereitstellen", "Projekt", "", Projekt),
    i(Cmd::TurboPruefen, "turbo_pruefen", "Projekt", "", Projekt),
    i(Cmd::TurboImDiskEditor, "turbo_im_disk_editor", "Projekt", "", Projekt),
    i(Cmd::ProjektKopieren, "projekt_kopieren", "Datei", "", Projekt),
    i(Cmd::DiskOeffnen, "disk_oeffnen", "Baum", "", Nichts),
    i(Cmd::ProjektOrdnerOeffnen, "projekt_ordner_oeffnen", "Datei", "", Nichts),
    i(Cmd::ProjektOeffnen, "projekt_oeffnen", "Datei", "Cmd+Umschalt+O", Nichts),
    i(Cmd::NeuerReiter, "neuer_reiter", "Datei", "Cmd+N", Nichts),
    i(Cmd::NeueNotiz, "neue_notiz", "Datei", "Cmd+Umschalt+T", Projekt),
    i(Cmd::DateiOeffnen, "datei_oeffnen", "Datei", "Cmd+O", Nichts),
    i(Cmd::Speichern, "speichern", "Datei", "Cmd+S", Nichts),   // immer erlaubt: ein gesperrter Menüpunkt verschluckt unter Windows die Taste, ohne sie weiterzugeben (nichts zu speichern: der Befehl tut nichts)
    i(Cmd::AlleSpeichern, "alle_speichern", "Datei", "Cmd+Alt+S", Nichts),
    i(Cmd::DateiSchliessen, "datei_schliessen", "Datei", "Cmd+W", Editor),
    i(Cmd::NeueDatei, "neue_datei", "Datei", "", Projekt),
    i(Cmd::NeuerOrdner, "neuer_ordner", "Baum", "", Projekt),
    i(Cmd::NeueNotizDialog, "neue_notiz_dialog", "Baum", "", Projekt),
    i(Cmd::ZielOeffnen, "ziel_oeffnen", "Baum", "", Projekt),
    i(Cmd::DateiUmbenennen, "datei_umbenennen", "Baum", "", Projekt),
    i(Cmd::DateiLoeschen, "datei_loeschen", "Baum", "", Projekt),
    i(Cmd::AlsInclude, "als_include", "Baum", "", Projekt),
    i(Cmd::PfadKopieren, "pfad_kopieren", "Baum", "", Projekt),
    i(Cmd::ImOrdnerZeigen, "im_ordner_zeigen", "Baum", "", Projekt),
    i(Cmd::BaumRueckgaengig, "baum_rueckgaengig", "Baum", "", Projekt),
    i(Cmd::HexAnsicht, "hex_ansicht", "Baum", "", Projekt),
    i(Cmd::Ausschneiden, "ausschneiden", "Bearbeiten", "", Editor),
    i(Cmd::Kopieren, "kopieren", "Bearbeiten", "", Editor),
    i(Cmd::Einfuegen, "einfuegen", "Bearbeiten", "", Editor),
    i(Cmd::GeheZuDefinition, "gehe_zu_definition", "Bearbeiten", "F12", Editor),
    i(Cmd::HaltepunktUmschalten, "haltepunkt_umschalten", "Bearbeiten", "F9", Editor),
    i(Cmd::Rueckgaengig, "rueckgaengig", "Bearbeiten", "Cmd+Z", Editor),
    i(Cmd::Wiederholen, "wiederholen", "Bearbeiten", "Cmd+Umschalt+Z", Editor),
    i(Cmd::Suchen, "suchen", "Bearbeiten", "Cmd+F", Editor),
    i(Cmd::Ersetzen, "ersetzen", "Bearbeiten", "Cmd+Alt+F", Editor),
    i(Cmd::GeheZuZeile, "gehe_zu_zeile", "Bearbeiten", "Cmd+L", Editor),
    i(Cmd::InProjektSuchen, "in_projekt_suchen", "Bearbeiten", "Cmd+Umschalt+F", Projekt),
    i(Cmd::Palette, "palette", "Ansicht", "Cmd+K", Nichts),
    i(Cmd::ProjektBereich, "projekt_bereich", "Ansicht", "Cmd+Alt+0", Nichts),
    i(Cmd::StudioVergroessern, "studio_vergroessern", "Ansicht", "", Projekt),
    i(Cmd::StudioVerkleinern, "studio_verkleinern", "Ansicht", "", Projekt),
    i(Cmd::StudioEinpassen, "studio_einpassen", "Ansicht", "", Projekt),
    i(Cmd::StudioRaster, "studio_raster", "Ansicht", "", Projekt),
    i(Cmd::StudioZwiebel, "studio_zwiebel", "Ansicht", "", Projekt),
    i(Cmd::TrackerNoten, "tracker_noten", "Ansicht", "Cmd+1", Projekt),
    i(Cmd::TrackerInstrumente, "tracker_instrumente", "Ansicht", "Cmd+2", Projekt),
    i(Cmd::TrackerAblauf, "tracker_ablauf", "Ansicht", "Cmd+3", Projekt),
    i(Cmd::Zen, "zen", "Ansicht", "Cmd+Alt+Z", Nichts),
    i(Cmd::Zahlenwerkzeuge, "zahlenwerkzeuge", "Bearbeiten", "", Nichts),
    i(Cmd::AlleAuswaehlen, "alle_auswaehlen", "Bearbeiten", "", Editor),
    i(Cmd::AllesFalten, "alles_falten", "Bearbeiten", "", Editor),
    i(Cmd::AllesEntfalten, "alles_entfalten", "Bearbeiten", "", Editor),
    i(Cmd::Vervollstaendigen, "vervollstaendigen", "Bearbeiten", "", Editor),
    i(Cmd::Korrigieren, "korrigieren", "Bearbeiten", "Cmd+.", Editor),
    i(Cmd::ZurKlammer, "zur_klammer", "Bearbeiten", "", Editor),
    i(Cmd::LeerzeichenEntfernen, "leerzeichen_entfernen", "Bearbeiten", "", Editor),
    i(Cmd::ZeilenSortieren, "zeilen_sortieren", "Bearbeiten", "", Editor),
    i(Cmd::KommentarUmschalten, "kommentar_umschalten", "Bearbeiten", "Cmd+Alt+C", Editor),
    i(Cmd::ProblemSpringen, "problem_springen", "Projekt", "", Nichts),
    i(Cmd::ProblemKopieren, "problem_kopieren", "Projekt", "", Nichts),
    i(Cmd::ProblemeAlleKopieren, "probleme_alle_kopieren", "Projekt", "", Nichts),
    i(Cmd::ProtokollKopieren, "protokoll_kopieren", "Projekt", "", Nichts),
    i(Cmd::ProtokollLeeren, "protokoll_leeren", "Projekt", "", Nichts),
    i(Cmd::ZeileFormatieren, "zeile_formatieren", "Bearbeiten", "", Editor),
    i(Cmd::DateiFormatieren, "datei_formatieren", "Bearbeiten", "", Editor),
    i(Cmd::BasicNachAsm, "basic_nach_asm", "Datei", "", Projekt),
    i(Cmd::MdUmschalten, "md_umschalten", "Ansicht", "", Projekt),
    i(Cmd::Seitenleiste, "seitenleiste", "Ansicht", "Cmd+B", Nichts),
    i(Cmd::Schublade, "schublade", "Ansicht", "Cmd+J", Nichts),
    i(Cmd::Panel, "panel", "Ansicht", "Cmd+Alt+B", Nichts),
    i(Cmd::ZoomPlus, "zoom_plus", "Ansicht", "Cmd+Plus", Nichts),
    i(Cmd::ZoomMinus, "zoom_minus", "Ansicht", "Cmd+Minus", Nichts),
    i(Cmd::ZoomNull, "zoom_null", "Ansicht", "Cmd+0", Nichts),
    i(Cmd::HellDunkel, "hell_dunkel", "Ansicht", "", Nichts),
    i(Cmd::Beenden, "beenden", "Datei", "Cmd+Q", Nichts),
    i(Cmd::Startseite, "startseite", "Datei", "", Nichts),
    i(Cmd::ProjektSchliessen, "projekt_schliessen", "Projekt", "Cmd+Umschalt+W", Projekt),
    i(Cmd::LesezeichenUmschalten, "lesezeichen_umschalten", "Bearbeiten", "Cmd+Alt+K", Editor),
    i(Cmd::NaechstesLesezeichen, "naechstes_lesezeichen", "Bearbeiten", "Cmd+Alt+J", Editor),
    i(Cmd::VorigesLesezeichen, "voriges_lesezeichen", "Bearbeiten", "Cmd+Alt+H", Editor),
    i(Cmd::Zeilenende, "zeilenende", "Bearbeiten", "", Editor),
    i(Cmd::Kodierung, "kodierung", "Bearbeiten", "", Editor),
    i(Cmd::HoherKontrast, "hoher_kontrast", "Ansicht", "", Nichts),
    i(Cmd::Umbruch, "umbruch", "Ansicht", "", Editor),
    i(Cmd::NaechsterReiter, "naechster_reiter", "Ansicht", "Cmd+Alt+Rechts", Nichts),
    i(Cmd::VorigerReiter, "voriger_reiter", "Ansicht", "Cmd+Alt+Links", Nichts),
    i(Cmd::ReiterAndereHaelfte, "reiter_andere_haelfte", "Ansicht", "", Editor),
    i(Cmd::Teilen, "teilen", "Ansicht", "Cmd+Alt+T", Editor),
    i(Cmd::ReiterLinks, "reiter_links", "Ansicht", "", Editor),
    i(Cmd::ReiterRechts, "reiter_rechts", "Ansicht", "", Editor),
    i(Cmd::MaximiereFokus, "maximiere_fokus", "Ansicht", "Cmd+Umschalt+Enter", Nichts),
    i(Cmd::SpalteMax, "spalte_max", "Ansicht", "", Nichts),
    i(Cmd::DokumentMax, "dokument_max", "Ansicht", "", Nichts),
    i(Cmd::SchubladeMax, "schublade_max", "Ansicht", "", Nichts),
    i(Cmd::PanelMax, "panel_max", "Ansicht", "", Nichts),
    i(Cmd::EditorSchriftGroesser, "editor_schrift_groesser", "Ansicht", "", Nichts),
    i(Cmd::EditorSchriftKleiner, "editor_schrift_kleiner", "Ansicht", "", Nichts),
    i(Cmd::DarstellungZuruecksetzen, "darstellung_zuruecksetzen", "Ansicht", "", Nichts),
    i(Cmd::LayoutZuruecksetzen, "layout_zuruecksetzen", "Ansicht", "", Nichts),
    i(Cmd::Einstellungen, "einstellungen", "Ansicht", "Cmd+Komma", Nichts),
    i(Cmd::Bauen, "bauen", "Projekt", "F7", Projekt),
    i(Cmd::Starten, "starten", "Projekt", "F5", Projekt),
    i(Cmd::Debuggen, "debuggen", "Projekt", "F6", Projekt),
    i(Cmd::BauenAktiv, "bauen_aktiv", "Projekt", "Alt+Umschalt+F7", Editor),
    i(Cmd::StartenAktiv, "starten_aktiv", "Projekt", "Alt+Umschalt+F5", Editor),
    i(Cmd::DebuggenAktiv, "debuggen_aktiv", "Projekt", "Alt+Umschalt+F6", Editor),
    i(Cmd::DebugWeiter, "debug_weiter", "Projekt", "Alt+F5", Projekt),
    i(Cmd::DebugPause, "debug_pause", "Projekt", "Alt+F6", Projekt),
    i(Cmd::DebugUeber, "debug_ueber", "Projekt", "F10", Projekt),
    i(Cmd::DebugHinein, "debug_hinein", "Projekt", "F11", Projekt),
    i(Cmd::DebugRaus, "debug_raus", "Projekt", "Umschalt+F11", Projekt),
    i(Cmd::DebugEnde, "debug_ende", "Projekt", "Umschalt+F5", Projekt),
    i(Cmd::DebugHierher, "debug_hierher", "Projekt", "", Editor),
    i(Cmd::TestsAusfuehren, "tests_ausfuehren", "Projekt", "Alt+F8", Projekt),
    i(Cmd::Exportieren, "exportieren", "Datei", "Cmd+Umschalt+E", Projekt),
    i(Cmd::PngLaden, "png_laden", "Bearbeiten", "", Projekt),
    i(Cmd::NaechstesProblem, "naechstes_problem", "Projekt", "F8", Projekt),
    i(Cmd::VorigesProblem, "voriges_problem", "Projekt", "Umschalt+F8", Projekt),
    i(Cmd::Hilfe, "hilfe", "Hilfe", "F1", Nichts),
    i(Cmd::BasicInLabels, "basic_in_labels", "Bearbeiten", "", Editor),
    i(Cmd::BasicInNummern, "basic_in_nummern", "Bearbeiten", "", Editor),
    i(Cmd::BasicNeuNummerieren, "basic_neu_nummerieren", "Bearbeiten", "", Editor),
];

/// Untermenüs der langen Menüs in der Reihenfolge der Anzeige (Menüleiste im Fenster und Menü des Macs); Datei und Hilfe bleiben flach
pub const UNTERMENUES: &[(&str, &[&str])] = &[
    ("Bearbeiten", &["Suchen und Ersetzen", "Markdown", "Zahlen und BASIC", "Falten", "Art und Kodierung", "Formatieren"]),
    ("Ansicht", &["Bereiche", "Zoom und Schrift", "Farbschema", "Studio", "Tracker", "Reiter und Teilen"]),
    ("Projekt", &["Probleme und Protokoll", "Debuggen", "Aktive Datei"]),
];
/// Befehle der Tabelle, die in dieser Version noch keine Funktion haben: Menü und Befehlspalette zeigen sie nicht (die Tabelle bleibt vollständig, damit Kennungen
/// und Katalogtexte stehen). Liste und Begründung: docs/befehle-ohne-funktion.md. Ein Befehl kommt hier heraus, sobald `ui/src/lib/commands.ts` ihn ausführt.
pub const NOCH_NICHT_GEBAUT: &[&str] = &["hex_vorschlaege", "std_zeichensatz", "zeichensatz_rom", "zeichensatz_png", "studio_vergroessern", "studio_verkleinern", "studio_einpassen",
    "studio_raster", "studio_zwiebel", "basic_nach_asm", "kodierung"];

/// Editor, ohne den der Befehl keinen Sinn hat (Kennung der Release-Auswahl: `sprite`, `tracker`, `disk` …); `None` = immer da
pub fn feature_von(c: Cmd) -> Option<&'static str> {
    use Cmd::*;
    match c { PngLaden => Some("sprite"), TrackerNoten | TrackerInstrumente | TrackerAblauf => Some("tracker"), _ => None }
}
/// Untermenü eines Befehls (leer: steht oben im Menü)
pub fn untermenue(c: Cmd) -> &'static str {
    use Cmd::*;
    match c {
        Suchen | Ersetzen | GeheZuZeile | InProjektSuchen | GeheZuDefinition | ZurKlammer | LesezeichenUmschalten | NaechstesLesezeichen | VorigesLesezeichen => "Suchen und Ersetzen",
        MdFett | MdKursiv | MdCode | MdDurchgestrichen | MdLink | MdBild | MdUeberschriftTiefer | MdUeberschriftHoeher | MdAufzaehlung | MdNummerierung | MdAufgaben | MdZitat | MdAufgabeUmschalten | MdTabelleNeu | MdTabelleAusrichten | MdAlsMarkdown => "Markdown",
        Zahlenwerkzeuge | HexVorschlaege | BasicInLabels | BasicInNummern | BasicNeuNummerieren => "Zahlen und BASIC",
        AllesFalten | AllesEntfalten => "Falten",
        ArtText | ArtAsm | ArtBasic | Zeilenende | Kodierung => "Art und Kodierung",
        ZeileFormatieren | DateiFormatieren | LeerzeichenEntfernen | ZeilenSortieren | KommentarUmschalten => "Formatieren",
        Seitenleiste | Schublade | Panel | ProjektBereich | MaximiereFokus | SpalteMax | DokumentMax | SchubladeMax | PanelMax | LayoutZuruecksetzen => "Bereiche",
        ZoomPlus | ZoomMinus | ZoomNull | EditorSchriftGroesser | EditorSchriftKleiner | DarstellungZuruecksetzen => "Zoom und Schrift",
        HellDunkel | HoherKontrast => "Farbschema",
        StudioVergroessern | StudioVerkleinern | StudioEinpassen | StudioRaster | StudioZwiebel => "Studio",
        TrackerNoten | TrackerInstrumente | TrackerAblauf => "Tracker",
        NaechsterReiter | VorigerReiter | ReiterAndereHaelfte | Teilen | ReiterLinks | ReiterRechts => "Reiter und Teilen",
        ProblemSpringen | ProblemKopieren | ProblemeAlleKopieren | ProtokollKopieren | ProtokollLeeren => "Probleme und Protokoll",
        DebugWeiter | DebugPause | DebugUeber | DebugHinein | DebugRaus | DebugEnde | DebugHierher => "Debuggen",
        BauenAktiv | StartenAktiv | DebuggenAktiv => "Aktive Datei",
        _ => "",
    }
}
/// Name im Menü: in einem Untermenü ohne dessen Namen als Vorspann („Markdown: Fett“ → „Fett“ im Untermenü Markdown); Befehlspalette und Tabelle behalten den vollen Namen
pub fn menuename(i: &Info) -> String { let u = untermenue(i.cmd); let n = i.name(); if u.is_empty() { n } else { n.strip_prefix(&format!("{}: ", untermenue_titel(u))).map(str::to_string).unwrap_or(n) } }
/// Titel eines Untermenüs in der aktuellen Sprache (`u` ist die deutsche Kennung aus [`UNTERMENUES`])
pub fn untermenue_titel(u: &str) -> String { vc_i18n::translate(&format!("submenu-{}", schluessel_teil(u)), &[]) }
pub fn schluessel_teil_pub(u: &str) -> String { schluessel_teil(u) }
fn schluessel_teil(u: &str) -> String {
    let t = u.to_lowercase().replace('ä', "ae").replace('ö', "oe").replace('ü', "ue").replace('ß', "ss");
    t.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty()).collect::<Vec<_>>().join("-")
}
/// Befehle eines Menüs (`unter` leer: die obersten Einträge) in Tabellenreihenfolge
pub fn menue_eintraege<'a>(menue: &'a str, unter: &'a str) -> impl Iterator<Item = &'static Info> + 'a { BEFEHLE.iter().filter(move |i| i.menue == menue && untermenue(i.cmd) == unter) }
/// Untermenüs eines Menüs (leer bei Datei und Hilfe)
pub fn untermenues(menue: &str) -> &'static [&'static str] { UNTERMENUES.iter().find(|(m, _)| *m == menue).map_or(&[], |(_, u)| *u) }

/// Menü „Baum“ steht bewusst nicht hier: Befehle, die nur im Kontextmenü des Dateibaums sinnvoll sind, bleiben dort und in der Befehlspalette, stehen aber in keiner Menüleiste.
pub const MENUES: &[&str] = &["Datei", "Bearbeiten", "Ansicht", "Projekt", "Hilfe"];
/// Angezeigter Titel eines Menüs in der aktuellen Sprache (die Kennung `m` aus [`MENUES`] bleibt deutsch und stabil)
pub fn menue_titel(m: &str) -> String {
    match m { "Datei" => vc_i18n::t!("menu-file"), "Bearbeiten" => vc_i18n::t!("menu-edit"), "Ansicht" => vc_i18n::t!("menu-view"), "Projekt" => vc_i18n::t!("menu-project"), "Hilfe" => vc_i18n::t!("menu-help"), "Baum" => vc_i18n::t!("menu-tree"), _ => m.to_string() }
}

/// Buchstaben für Alt+Buchstabe (Windows und Linux), aus den **übersetzten** Menütiteln abgeleitet: je Menü der erste Buchstabe des Titels,
/// der in der Menüleiste noch nicht vergeben ist. So bleibt jedes Kürzel eindeutig und steht im sichtbaren Titel (wird dort unterstrichen).
pub fn menue_buchstaben() -> Vec<(&'static str, char)> {
    let mut vergeben: Vec<char> = vec![]; let mut out = vec![];
    for m in MENUES {
        let titel = menue_titel(m);
        if let Some(c) = titel.chars().filter(|c| c.is_alphabetic()).flat_map(|c| c.to_lowercase()).find(|c| !vergeben.contains(c)) { vergeben.push(c); out.push((*m, c)); }
    }
    out
}
/// Alt-Buchstabe eines Menüs (Kennung aus [`MENUES`]) in der aktuellen Sprache
pub fn menue_buchstabe(m: &str) -> Option<char> { menue_buchstaben().into_iter().find(|(k, _)| *k == m).map(|(_, c)| c) }

pub fn info(c: Cmd) -> &'static Info { BEFEHLE.iter().find(|i| i.cmd == c).expect("Befehl fehlt in BEFEHLE") }
pub fn nach_id(id: &str) -> Option<Cmd> { BEFEHLE.iter().find(|i| i.id == id).map(|i| i.cmd) }

/// Kürzel eines Befehls auf dieser Plattform: F10, F11 und Umschalt+F11 sind am Mac vom System belegt (Lautstärke, Desktop anzeigen),
/// dort gelten für die Debug-Schritte Cmd+Alt+O (über), Cmd+Alt+I (hinein) und Cmd+Alt+U (hinaus); sonst das Kürzel der Tabelle
pub fn tasten_fuer<'a>(id: &str, tasten: &'a str, mac: bool) -> &'a str {
    if !mac { return tasten }
    match id { "debug_ueber" => "Cmd+Alt+O", "debug_hinein" => "Cmd+Alt+I", "debug_raus" => "Cmd+Alt+U", _ => tasten }
}

/// Kürzel zerlegt: (Cmd, Umschalt, Alt, Taste)
pub fn zerlege(tasten: &str) -> Option<(bool, bool, bool, &str)> {
    if tasten.is_empty() { return None }
    let mut p: Vec<&str> = tasten.split('+').collect(); let taste = p.pop()?;
    Some((p.contains(&"Cmd"), p.contains(&"Umschalt"), p.contains(&"Alt"), taste))
}

/// Anzeige je Plattform: macOS ⌘⇧⌥, sonst Strg+Umschalt+Alt
pub fn fuer_anzeige(tasten: &str, mac: bool) -> String {
    if mac { tasten.replace("Cmd+", "⌘").replace("Umschalt+", "⇧").replace("Alt+", "⌥").replace("Plus", "+").replace("Minus", "−").replace("Komma", ",").replace("Rechts", "→").replace("Links", "←") }
    else { tasten.replace("Cmd", &vc_i18n::t!("key-ctrl")).replace("Umschalt", &vc_i18n::t!("key-shift")).replace("Komma", ",").replace("Rechts", &vc_i18n::t!("key-right")).replace("Links", &vc_i18n::t!("key-left")) }
}

/// Freier Text mit Tastenangaben („Rückgängig (Cmd+Z)“, „Cmd/Strg+Z“): auf Windows und Linux wird „Cmd“ zu „Strg“, auf macOS bleibt der Text.
pub fn plattform_text(text: &str) -> String { plattform_text_fuer(text, cfg!(target_os = "macos")) }
pub fn plattform_text_fuer(text: &str, mac: bool) -> String { if mac || !text.contains("Cmd") { text.to_string() } else { let c = vc_i18n::t!("key-ctrl"); text.replace("Cmd/Strg", &c).replace("Strg/Cmd", &c).replace("Cmd/Ctrl", &c).replace("Ctrl/Cmd", &c).replace("Cmd", &c) } }

/// Unscharfe Suche für die Palette: alle Wörter müssen vorkommen (Groß/Klein egal), Treffer am Wortanfang zuerst
pub fn filtere(frage: &str) -> Vec<&'static Info> {
    let q = frage.to_lowercase(); let w: Vec<&str> = q.split_whitespace().collect();
    let mut v: Vec<(u8, &Info)> = BEFEHLE.iter().filter_map(|i| { let n = i.name().to_lowercase(); if w.iter().all(|x| n.contains(x)) { Some((if w.first().map_or(true, |x| n.starts_with(x)) { 0 } else { 1 }, i)) } else { None } }).collect();
    v.sort_by_key(|x| x.0); v.into_iter().map(|x| x.1).collect()
}

#[cfg(test)]
static SPERRE: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// Tests, die die dynamischen Befehle (globaler Zustand) ändern, laufen nacheinander
/// Für Tests anderer Crates (dynamischer Befehlszustand ist global)
pub fn test_sperre_pub() -> std::sync::MutexGuard<'static, ()> { SPERRE_PUB.lock().unwrap_or_else(|e| e.into_inner()) }
static SPERRE_PUB: std::sync::Mutex<()> = std::sync::Mutex::new(());
#[cfg(test)]
pub fn test_sperre() -> std::sync::MutexGuard<'static, ()> { SPERRE.lock().unwrap_or_else(|e| e.into_inner()) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test] fn mac_kuerzel_der_debug_schritte() {
        for (id, mac_k) in [("debug_ueber", "Cmd+Alt+O"), ("debug_hinein", "Cmd+Alt+I"), ("debug_raus", "Cmd+Alt+U")] {
            let i = BEFEHLE.iter().find(|i| i.id == id).unwrap();
            assert_eq!(tasten_fuer(id, i.tasten, true), mac_k); assert_eq!(tasten_fuer(id, i.tasten, false), i.tasten);
            assert!(!BEFEHLE.iter().any(|x| x.tasten == mac_k), "{mac_k} ist schon belegt");
        }
        assert_eq!(tasten_fuer("bauen", "F7", true), "F7");
    }
    #[test] fn menues_stimmen_und_jeder_befehl_hat_ein_menue() {
        for i in BEFEHLE { assert!(MENUES.contains(&i.menue) || i.menue == "Baum", "{} hat unbekanntes Menü {}", i.name(), i.menue) }
        // Untermenüs: jeder Befehl eines Menüs steht genau einmal (oben oder in genau einem erklärten Untermenü), kein leeres Untermenü
        for i in BEFEHLE.iter().filter(|i| MENUES.contains(&i.menue)) { let u = untermenue(i.cmd); assert!(u.is_empty() || untermenues(i.menue).contains(&u), "{}: Untermenü „{}“ gehört nicht zu {}", i.name(), u, i.menue) }
        for (m, subs) in UNTERMENUES { for u in *subs { assert!(menue_eintraege(m, u).count() > 0, "leeres Untermenü {m} ▸ {u}") } }
        for m in MENUES { let n = menue_eintraege(m, "").count() + untermenues(m).iter().map(|u| menue_eintraege(m, u).count()).sum::<usize>(); assert_eq!(n, BEFEHLE.iter().filter(|i| i.menue == *m).count(), "{m}: jeder Befehl genau einmal") }
        for m in ["Bearbeiten", "Ansicht", "Projekt"] { assert!(menue_eintraege(m, "").count() + untermenues(m).len() <= 18, "{m}: oberste Ebene zu lang") }
        // Menü „Datei“ bleibt übersichtlich; die Baum-Befehle stehen im Kontextmenü des Dateibaums
        assert!(BEFEHLE.iter().filter(|i| i.menue == "Datei").count() <= 18, "Menü Datei zu lang");
        for c in [Cmd::DateiUmbenennen, Cmd::DateiLoeschen, Cmd::PfadKopieren, Cmd::ImOrdnerZeigen, Cmd::HexAnsicht, Cmd::AlsInclude, Cmd::ZielOeffnen, Cmd::NeuerOrdner, Cmd::NeueNotizDialog] { assert_eq!(info(c).menue, "Baum", "{:?}", c) }
        for m in MENUES { assert!(BEFEHLE.iter().any(|i| i.menue == *m), "leeres Menü {m}") }
    }
    #[test] fn jeder_befehl_und_jedes_untermenue_hat_deutsche_und_englische_texte() {
        use vc_i18n::{has_key_in, Lang};
        for l in [Lang::De, Lang::En] {
            for i in BEFEHLE { assert!(has_key_in(l, &format!("cmd-{}", i.id)), "{}: Name von {} fehlt", l.code(), i.id) }
            for (_, subs) in UNTERMENUES { for u in *subs { assert!(has_key_in(l, &format!("submenu-{}", schluessel_teil(u))), "{}: Untermenü {u} fehlt", l.code()) } }
        }
    }
    #[test] fn alt_buchstaben_der_menues_sind_in_jeder_sprache_eindeutig_und_stehen_im_titel() {
        for l in [vc_i18n::Lang::De, vc_i18n::Lang::En] {
            vc_i18n::set_thread_language(Some(l));
            let b = menue_buchstaben(); assert_eq!(b.len(), MENUES.len(), "{}: jedes Menü hat einen Buchstaben", l.code());
            let mut s: Vec<char> = b.iter().map(|x| x.1).collect(); s.sort(); s.dedup(); assert_eq!(s.len(), b.len(), "{}: Buchstaben doppelt: {b:?}", l.code());
            for (m, c) in &b { assert!(menue_titel(m).to_lowercase().contains(*c), "{}: {c} steht nicht in {}", l.code(), menue_titel(m)) }
            if l == vc_i18n::Lang::De { assert_eq!(b.iter().map(|x| x.1).collect::<String>(), "dbaph", "deutsche Belegung wie bisher") }
        }
        vc_i18n::set_thread_language(None);
    }
    #[test] fn kuerzel_sind_eindeutig() {
        let mut seen: HashMap<&str, String> = HashMap::new();
        for i in BEFEHLE.iter().filter(|i| !i.tasten.is_empty()) { if let Some(o) = seen.insert(i.tasten, i.name()) { panic!("{} und {} teilen {}", o, i.name(), i.tasten) } }
    }
    #[test] fn kuerzel_gehoeren_nicht_der_textbearbeitung() {
        // Ausschneiden, Kopieren, Einfügen, Alles auswählen bleiben dem Editor; Cmd+B ist die Seitenleiste, daher kein Fett
        for k in ["Cmd+X", "Cmd+C", "Cmd+V", "Cmd+A"] { assert!(!BEFEHLE.iter().any(|i| i.tasten == k), "{k} ist reserviert") }
    }
    #[test] fn ids_sind_eindeutig_und_findbar() {
        let mut ids: Vec<&str> = BEFEHLE.iter().map(|i| i.id).collect(); ids.sort(); let n = ids.len(); ids.dedup(); assert_eq!(n, ids.len());
        assert_eq!(nach_id("bauen"), Some(Cmd::Bauen)); assert_eq!(info(Cmd::Palette).tasten, "Cmd+K");
    }
    #[test] fn kuerzel_zerlegen_und_anzeigen() {
        assert_eq!(zerlege("Cmd+Umschalt+F"), Some((true, true, false, "F"))); assert_eq!(zerlege("F7"), Some((false, false, false, "F7")));
        assert_eq!(fuer_anzeige("Cmd+Umschalt+F", true), "⌘⇧F"); assert_eq!(fuer_anzeige("Cmd+S", false), "Strg+S");
        assert_eq!(plattform_text_fuer("Auch mit Cmd + und −, zurück mit Cmd 0 (Cmd/Strg+Z)", false), "Auch mit Strg + und −, zurück mit Strg 0 (Strg+Z)"); assert_eq!(plattform_text_fuer("Rückgängig (Cmd+Z)", true), "Rückgängig (Cmd+Z)");
    }
    #[test] fn palette_filtert() {
        assert_eq!(filtere("bauen")[0].cmd, Cmd::Bauen); assert!(filtere("zzz").is_empty()); assert!(filtere("").len() == BEFEHLE.len());
        assert!(filtere("projekt suchen").iter().any(|i| i.cmd == Cmd::InProjektSuchen));
    }
}
