// SPDX-License-Identifier: GPL-3.0-or-later
//! Die **eine** Befehlstabelle: Menü, Befehlspalette und Kürzel entstehen alle hieraus (Konzept Abschnitt 7).
//! Kürzel stehen plattformneutral als „Cmd+Umschalt+S“; `fuer_anzeige` macht daraus ⌘⇧S (macOS) oder Strg+Umschalt+S.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cmd {
    ProjektOeffnen, NeuerReiter, NeueNotiz, DateiOeffnen, Speichern, AlleSpeichern, DateiSchliessen,
    Rueckgaengig, Wiederholen, Suchen, Ersetzen, GeheZuZeile, InProjektSuchen,
    Palette, StudioVergroessern, StudioVerkleinern, StudioEinpassen, StudioRaster, StudioZwiebel, TrackerNoten, TrackerInstrumente, TrackerAblauf, Seitenleiste, Schublade, Panel, Zen, MdUmschalten, BasicNachAsm, Zahlenwerkzeuge, ZoomPlus, ZoomMinus, ZoomNull, AlsInclude, AlleAuswaehlen, AllesFalten, AllesEntfalten, Vervollstaendigen, ProblemSpringen, ProblemKopieren, ProblemeAlleKopieren, ProtokollKopieren, ProtokollLeeren, ZeileFormatieren, DateiFormatieren, EditorSchriftGroesser, EditorSchriftKleiner, DarstellungZuruecksetzen, HellDunkel, HoherKontrast, Umbruch, Einstellungen, Startseite, Zeilenende, Kodierung, Beenden, ProjektNeu, Assistent, Ueber, Feedback, UpdatePruefen, ProjektOrdnerOeffnen, ProjektKopieren,  ProjektEinstellungen,
    Bauen, Starten, Debuggen, BauenAktiv, StartenAktiv, DebuggenAktiv, MdFett, MdKursiv, MdCode, MdDurchgestrichen, MdLink, MdBild, MdUeberschriftTiefer, MdUeberschriftHoeher, MdAufzaehlung, MdNummerierung, MdAufgaben, MdZitat, MdAufgabeUmschalten, MdTabelleNeu, MdTabelleAusrichten, MdAlsMarkdown, HilfeImReiter, ArtText, ArtAsm, ArtBasic, HexVorschlaege, StdZeichensatz, ZeichensatzAusRom, ZeichensatzAusPng, AssistentRoms, AssistentEmulatoren, AssistentHardware, AssistentDesign, AssistentNeues, DebugWeiter, DebugPause, DebugUeber, DebugHinein, DebugRaus, DebugEnde, DebugHierher, TestsAusfuehren, Exportieren, NaechstesProblem, VorigesProblem,
    NaechsterReiter, VorigerReiter, NeueDatei, NeuerOrdner, NeueNotizDialog, ZielOeffnen, DateiUmbenennen, DateiLoeschen, PfadKopieren, ImOrdnerZeigen, HexAnsicht, BaumRueckgaengig, Ausschneiden, Kopieren, Einfuegen, GeheZuDefinition, HaltepunktUmschalten, ReiterAndereHaelfte, Teilen, ReiterLinks, ReiterRechts, MaximiereFokus, SpalteMax, DokumentMax, SchubladeMax, PanelMax, LayoutZuruecksetzen, Hilfe,
}

/// Wann ein Befehl nutzbar ist (sonst ausgegraut, mit Grund)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Braucht { Nichts, Projekt, Editor }

pub struct Info { pub cmd: Cmd, pub menue: &'static str, pub name: &'static str, pub tasten: &'static str, pub braucht: Braucht, pub id: &'static str }

const fn i(cmd: Cmd, id: &'static str, menue: &'static str, name: &'static str, tasten: &'static str, braucht: Braucht) -> Info { Info { cmd, menue, name, tasten, braucht, id } }
use Braucht::*;

pub const BEFEHLE: &[Info] = &[
    i(Cmd::Ueber, "ueber", "Hilfe", "Über VintageCoder", "", Nichts),
    i(Cmd::Feedback, "feedback", "Hilfe", "Feedback senden …", "", Nichts),
    i(Cmd::UpdatePruefen, "update_pruefen", "Hilfe", "Nach Updates suchen …", "", Nichts),
    i(Cmd::Assistent, "assistent", "Hilfe", "Konfigurationsassistent …", "", Nichts),
    i(Cmd::MdFett, "md_fett", "Bearbeiten", "Markdown: Fett", "", Editor),
    i(Cmd::MdKursiv, "md_kursiv", "Bearbeiten", "Markdown: Kursiv", "Cmd+I", Editor),
    i(Cmd::MdCode, "md_code", "Bearbeiten", "Markdown: Code", "Cmd+E", Editor),
    i(Cmd::MdDurchgestrichen, "md_durchgestrichen", "Bearbeiten", "Markdown: Durchgestrichen", "Cmd+Umschalt+X", Editor),
    i(Cmd::MdLink, "md_link", "Bearbeiten", "Markdown: Link einfügen", "Cmd+Umschalt+K", Editor),
    i(Cmd::MdBild, "md_bild", "Bearbeiten", "Markdown: Bild einfügen …", "", Editor),
    i(Cmd::MdUeberschriftTiefer, "md_tiefer", "Bearbeiten", "Markdown: Überschrift eine Ebene tiefer (# ergänzen)", "", Editor),
    i(Cmd::MdUeberschriftHoeher, "md_hoeher", "Bearbeiten", "Markdown: Überschrift eine Ebene höher (# entfernen)", "", Editor),
    i(Cmd::MdAufzaehlung, "md_aufzaehlung", "Bearbeiten", "Markdown: Aufzählung", "", Editor),
    i(Cmd::MdNummerierung, "md_nummerierung", "Bearbeiten", "Markdown: Nummerierung", "", Editor),
    i(Cmd::MdAufgaben, "md_aufgaben", "Bearbeiten", "Markdown: Aufgabenliste", "", Editor),
    i(Cmd::MdZitat, "md_zitat", "Bearbeiten", "Markdown: Zitat", "", Editor),
    i(Cmd::MdAufgabeUmschalten, "md_aufgabe", "Bearbeiten", "Markdown: Aufgabe abhaken / wieder öffnen", "", Editor),
    i(Cmd::MdTabelleNeu, "md_tabelle", "Bearbeiten", "Markdown: Tabelle einfügen", "", Editor),
    i(Cmd::MdTabelleAusrichten, "md_ausrichten", "Bearbeiten", "Markdown: Tabelle ausrichten", "", Editor),
    i(Cmd::MdAlsMarkdown, "md_als", "Bearbeiten", "Markdown: Datei als Markdown behandeln / als Text", "", Editor),
    i(Cmd::HilfeImReiter, "hilfe_reiter", "Hilfe", "Hilfe im Reiter öffnen", "", Nichts),
    i(Cmd::ArtText, "art_text", "Bearbeiten", "Art: Text", "", Editor),
    i(Cmd::ArtAsm, "art_asm", "Bearbeiten", "Art: Assembler", "", Editor),
    i(Cmd::ArtBasic, "art_basic", "Bearbeiten", "Art: BASIC", "", Editor),
    i(Cmd::HexVorschlaege, "hex_vorschlaege", "Bearbeiten", "BASIC: Hex-Vorschläge für Adressen …", "", Editor),
    i(Cmd::StdZeichensatz, "std_zeichensatz", "Datei", "Standardzeichensatz hinzufügen …", "", Projekt),
    i(Cmd::ZeichensatzAusRom, "zeichensatz_rom", "Datei", "Zeichensatz aus ROM importieren …", "", Projekt),
    i(Cmd::ZeichensatzAusPng, "zeichensatz_png", "Datei", "Zeichensatz aus PNG-Raster importieren …", "", Projekt),
    i(Cmd::AssistentNeues, "assistent_neues", "Hilfe", "Konfigurationsassistent: Nach Neuem suchen …", "", Nichts),
    i(Cmd::AssistentRoms, "assistent_roms", "Hilfe", "Konfigurationsassistent: nur ROMs …", "", Nichts),
    i(Cmd::AssistentEmulatoren, "assistent_emulatoren", "Hilfe", "Konfigurationsassistent: nur Emulatoren …", "", Nichts),
    i(Cmd::AssistentHardware, "assistent_hardware", "Hilfe", "Konfigurationsassistent: nur echte Hardware …", "", Nichts),
    i(Cmd::AssistentDesign, "assistent_design", "Hilfe", "Konfigurationsassistent: nur Design und Editor …", "", Nichts),
    i(Cmd::ProjektNeu, "projekt_neu", "Datei", "Neues Projekt …", "Cmd+Alt+N", Nichts),
    i(Cmd::ProjektEinstellungen, "projekt_einstellungen", "Projekt", "Projekt-Einstellungen …", "", Projekt),
    i(Cmd::ProjektKopieren, "projekt_kopieren", "Datei", "Projekt kopieren …", "", Projekt),
    i(Cmd::ProjektOrdnerOeffnen, "projekt_ordner_oeffnen", "Datei", "Projektordner öffnen …", "", Nichts),
    i(Cmd::ProjektOeffnen, "projekt_oeffnen", "Datei", "Projekt öffnen … (Projektdatei)", "Cmd+Umschalt+O", Nichts),
    i(Cmd::NeuerReiter, "neuer_reiter", "Datei", "Neuer Reiter (unbenannt)", "Cmd+N", Nichts),
    i(Cmd::NeueNotiz, "neue_notiz", "Datei", "Neue Notiz", "Cmd+Umschalt+T", Projekt),
    i(Cmd::DateiOeffnen, "datei_oeffnen", "Datei", "Datei öffnen …", "Cmd+O", Nichts),
    i(Cmd::Speichern, "speichern", "Datei", "Speichern", "Cmd+S", Editor),
    i(Cmd::AlleSpeichern, "alle_speichern", "Datei", "Alle speichern", "Cmd+Alt+S", Projekt),
    i(Cmd::DateiSchliessen, "datei_schliessen", "Datei", "Datei schließen", "Cmd+W", Editor),
    i(Cmd::NeueDatei, "neue_datei", "Datei", "Neue Datei …", "", Projekt),
    i(Cmd::NeuerOrdner, "neuer_ordner", "Baum", "Neuer Ordner …", "", Projekt),
    i(Cmd::NeueNotizDialog, "neue_notiz_dialog", "Baum", "Neue Notiz …", "", Projekt),
    i(Cmd::ZielOeffnen, "ziel_oeffnen", "Baum", "Öffnen", "", Projekt),
    i(Cmd::DateiUmbenennen, "datei_umbenennen", "Baum", "Umbenennen …", "", Projekt),
    i(Cmd::DateiLoeschen, "datei_loeschen", "Baum", "Löschen …", "", Projekt),
    i(Cmd::AlsInclude, "als_include", "Baum", "Als Include einfügen", "", Projekt),
    i(Cmd::PfadKopieren, "pfad_kopieren", "Baum", "Pfad kopieren", "", Projekt),
    i(Cmd::ImOrdnerZeigen, "im_ordner_zeigen", "Baum", "Im Ordner zeigen", "", Projekt),
    i(Cmd::BaumRueckgaengig, "baum_rueckgaengig", "Baum", "Verschieben im Baum rückgängig", "", Projekt),
    i(Cmd::HexAnsicht, "hex_ansicht", "Baum", "Als Hex ansehen (nur Lesen)", "", Projekt),
    i(Cmd::Ausschneiden, "ausschneiden", "Bearbeiten", "Ausschneiden", "", Editor),
    i(Cmd::Kopieren, "kopieren", "Bearbeiten", "Kopieren", "", Editor),
    i(Cmd::Einfuegen, "einfuegen", "Bearbeiten", "Einfügen", "", Editor),
    i(Cmd::GeheZuDefinition, "gehe_zu_definition", "Bearbeiten", "Gehe zu Definition", "F12", Editor),
    i(Cmd::HaltepunktUmschalten, "haltepunkt_umschalten", "Bearbeiten", "Haltepunkt umschalten", "F9", Editor),
    i(Cmd::Rueckgaengig, "rueckgaengig", "Bearbeiten", "Rückgängig", "Cmd+Z", Editor),
    i(Cmd::Wiederholen, "wiederholen", "Bearbeiten", "Wiederholen", "Cmd+Umschalt+Z", Editor),
    i(Cmd::Suchen, "suchen", "Bearbeiten", "Suchen …", "Cmd+F", Editor),
    i(Cmd::Ersetzen, "ersetzen", "Bearbeiten", "Ersetzen …", "Cmd+Alt+F", Editor),
    i(Cmd::GeheZuZeile, "gehe_zu_zeile", "Bearbeiten", "Gehe zu Zeile …", "Cmd+L", Editor),
    i(Cmd::InProjektSuchen, "in_projekt_suchen", "Bearbeiten", "In Projekt suchen …", "Cmd+Umschalt+F", Projekt),
    i(Cmd::Palette, "palette", "Ansicht", "Befehl oder Datei suchen …", "Cmd+K", Nichts),
    i(Cmd::StudioVergroessern, "studio_vergroessern", "Ansicht", "Studio: Fläche vergrößern", "", Projekt),
    i(Cmd::StudioVerkleinern, "studio_verkleinern", "Ansicht", "Studio: Fläche verkleinern", "", Projekt),
    i(Cmd::StudioEinpassen, "studio_einpassen", "Ansicht", "Studio: Fläche einpassen", "", Projekt),
    i(Cmd::StudioRaster, "studio_raster", "Ansicht", "Studio: Raster ein-/ausschalten", "", Projekt),
    i(Cmd::StudioZwiebel, "studio_zwiebel", "Ansicht", "Studio: Zwiebelhaut ein-/ausschalten", "", Projekt),
    i(Cmd::TrackerNoten, "tracker_noten", "Ansicht", "Tracker: Noten", "Cmd+1", Projekt),
    i(Cmd::TrackerInstrumente, "tracker_instrumente", "Ansicht", "Tracker: Instrumente", "Cmd+2", Projekt),
    i(Cmd::TrackerAblauf, "tracker_ablauf", "Ansicht", "Tracker: Ablauf", "Cmd+3", Projekt),
    i(Cmd::Zen, "zen", "Ansicht", "Zen-Modus: nur die Fläche (Seitenleiste, Liste, Seitenspalte, Schublade ausblenden)", "Cmd+Umschalt+B", Nichts),
    i(Cmd::Zahlenwerkzeuge, "zahlenwerkzeuge", "Bearbeiten", "Zahlenwerkzeuge …", "", Nichts),
    i(Cmd::AlleAuswaehlen, "alle_auswaehlen", "Bearbeiten", "Alles auswählen", "", Editor),
    i(Cmd::AllesFalten, "alles_falten", "Bearbeiten", "Alles falten", "", Editor),
    i(Cmd::AllesEntfalten, "alles_entfalten", "Bearbeiten", "Alles entfalten", "", Editor),
    i(Cmd::Vervollstaendigen, "vervollstaendigen", "Bearbeiten", "Vervollständigung öffnen", "", Editor),
    i(Cmd::ProblemSpringen, "problem_springen", "Projekt", "Problem: Zur Zeile springen", "", Nichts),
    i(Cmd::ProblemKopieren, "problem_kopieren", "Projekt", "Problem: Meldung kopieren", "", Nichts),
    i(Cmd::ProblemeAlleKopieren, "probleme_alle_kopieren", "Projekt", "Alle Probleme kopieren", "", Nichts),
    i(Cmd::ProtokollKopieren, "protokoll_kopieren", "Projekt", "Protokoll kopieren", "", Nichts),
    i(Cmd::ProtokollLeeren, "protokoll_leeren", "Projekt", "Protokoll leeren (Ausgabe leeren)", "", Nichts),
    i(Cmd::ZeileFormatieren, "zeile_formatieren", "Bearbeiten", "Zeile formatieren", "", Editor),
    i(Cmd::DateiFormatieren, "datei_formatieren", "Bearbeiten", "Datei formatieren", "", Editor),
    i(Cmd::BasicNachAsm, "basic_nach_asm", "Datei", "BASIC in Assembler umwandeln", "", Projekt),
    i(Cmd::MdUmschalten, "md_umschalten", "Ansicht", "Markdown: Vorschau / Quelltext umschalten", "", Projekt),
    i(Cmd::Seitenleiste, "seitenleiste", "Ansicht", "Linke Spalte ein-/ausklappen", "Cmd+B", Nichts),
    i(Cmd::Schublade, "schublade", "Ansicht", "Schublade ein-/ausklappen (Probleme, Protokoll)", "Cmd+J", Nichts),
    i(Cmd::Panel, "panel", "Ansicht", "Rechtes Panel ein-/ausklappen", "Cmd+Alt+B", Nichts),
    i(Cmd::ZoomPlus, "zoom_plus", "Ansicht", "Oberfläche vergrößern", "Cmd+Plus", Nichts),
    i(Cmd::ZoomMinus, "zoom_minus", "Ansicht", "Oberfläche verkleinern", "Cmd+Minus", Nichts),
    i(Cmd::ZoomNull, "zoom_null", "Ansicht", "Oberfläche: Originalgröße", "Cmd+0", Nichts),
    i(Cmd::HellDunkel, "hell_dunkel", "Ansicht", "Hell/Dunkel umschalten", "", Nichts),
    i(Cmd::Beenden, "beenden", "Datei", "VintageCoder beenden", "Cmd+Q", Nichts),
    i(Cmd::Startseite, "startseite", "Datei", "Startseite", "", Nichts),
    i(Cmd::Zeilenende, "zeilenende", "Bearbeiten", "Zeilenende umschalten (LF/CRLF)", "", Editor),
    i(Cmd::Kodierung, "kodierung", "Bearbeiten", "Kodierung umschalten (UTF-8/ISO-8859-1)", "", Editor),
    i(Cmd::HoherKontrast, "hoher_kontrast", "Ansicht", "Hoher Kontrast ein/aus", "", Nichts),
    i(Cmd::Umbruch, "umbruch", "Ansicht", "Zeilenumbruch im Editor", "", Editor),
    i(Cmd::NaechsterReiter, "naechster_reiter", "Ansicht", "Nächster Reiter", "Cmd+Alt+Rechts", Nichts),
    i(Cmd::VorigerReiter, "voriger_reiter", "Ansicht", "Vorheriger Reiter", "Cmd+Alt+Links", Nichts),
    i(Cmd::ReiterAndereHaelfte, "reiter_andere_haelfte", "Ansicht", "Reiter in die andere Hälfte schicken", "", Editor),
    i(Cmd::Teilen, "teilen", "Ansicht", "Editor teilen / Teilung aufheben", "Cmd+Alt+T", Editor),
    i(Cmd::ReiterLinks, "reiter_links", "Ansicht", "Reiter nach links verschieben", "", Editor),
    i(Cmd::ReiterRechts, "reiter_rechts", "Ansicht", "Reiter nach rechts verschieben", "", Editor),
    i(Cmd::MaximiereFokus, "maximiere_fokus", "Ansicht", "Bereich mit Fokus maximieren / wiederherstellen", "Cmd+Umschalt+Enter", Nichts),
    i(Cmd::SpalteMax, "spalte_max", "Ansicht", "Linke Spalte maximieren / wiederherstellen", "", Nichts),
    i(Cmd::DokumentMax, "dokument_max", "Ansicht", "Dokumentenfeld maximieren / wiederherstellen", "", Nichts),
    i(Cmd::SchubladeMax, "schublade_max", "Ansicht", "Schublade maximieren / wiederherstellen", "", Nichts),
    i(Cmd::PanelMax, "panel_max", "Ansicht", "Panel maximieren / wiederherstellen", "", Nichts),
    i(Cmd::EditorSchriftGroesser, "editor_schrift_groesser", "Ansicht", "Editor-Schrift größer", "", Nichts),
    i(Cmd::EditorSchriftKleiner, "editor_schrift_kleiner", "Ansicht", "Editor-Schrift kleiner", "", Nichts),
    i(Cmd::DarstellungZuruecksetzen, "darstellung_zuruecksetzen", "Ansicht", "Darstellung zurücksetzen (Größe, Schrift, Layout)", "", Nichts),
    i(Cmd::LayoutZuruecksetzen, "layout_zuruecksetzen", "Ansicht", "Layout zurücksetzen", "", Nichts),
    i(Cmd::Einstellungen, "einstellungen", "Ansicht", "Einstellungen", "Cmd+Komma", Nichts),
    i(Cmd::Bauen, "bauen", "Projekt", "Projekt bauen", "F7", Projekt),
    i(Cmd::Starten, "starten", "Projekt", "Startprogramm starten", "F5", Projekt),
    i(Cmd::Debuggen, "debuggen", "Projekt", "Startprogramm debuggen", "F6", Projekt),
    i(Cmd::BauenAktiv, "bauen_aktiv", "Projekt", "Aktive Datei bauen", "Alt+Umschalt+F7", Editor),
    i(Cmd::StartenAktiv, "starten_aktiv", "Projekt", "Aktive Datei starten", "Alt+Umschalt+F5", Editor),
    i(Cmd::DebuggenAktiv, "debuggen_aktiv", "Projekt", "Aktive Datei debuggen", "Alt+Umschalt+F6", Editor),
    i(Cmd::DebugWeiter, "debug_weiter", "Projekt", "Debuggen: Weiter", "Alt+F5", Projekt),
    i(Cmd::DebugPause, "debug_pause", "Projekt", "Debuggen: Anhalten", "Alt+F6", Projekt),
    i(Cmd::DebugUeber, "debug_ueber", "Projekt", "Debuggen: Schritt über", "F10", Projekt),
    i(Cmd::DebugHinein, "debug_hinein", "Projekt", "Debuggen: Schritt hinein", "F11", Projekt),
    i(Cmd::DebugRaus, "debug_raus", "Projekt", "Debuggen: Schritt hinaus", "Umschalt+F11", Projekt),
    i(Cmd::DebugEnde, "debug_ende", "Projekt", "Debuggen beenden", "Umschalt+F5", Projekt),
    i(Cmd::DebugHierher, "debug_hierher", "Projekt", "Debuggen: Bis hierher ausführen", "", Editor),
    i(Cmd::TestsAusfuehren, "tests_ausfuehren", "Projekt", "Tests ausführen", "Alt+F8", Projekt),
    i(Cmd::Exportieren, "exportieren", "Datei", "Exportieren …", "Cmd+Umschalt+E", Projekt),
    i(Cmd::NaechstesProblem, "naechstes_problem", "Projekt", "Nächstes Problem", "F8", Projekt),
    i(Cmd::VorigesProblem, "voriges_problem", "Projekt", "Voriges Problem", "Umschalt+F8", Projekt),
    i(Cmd::Hilfe, "hilfe", "Hilfe", "Hilfe", "F1", Nichts),
];

/// Untermenüs der langen Menüs in der Reihenfolge der Anzeige (Menüleiste im Fenster und Menü des Macs); Datei und Hilfe bleiben flach
pub const UNTERMENUES: &[(&str, &[&str])] = &[
    ("Bearbeiten", &["Suchen und Ersetzen", "Markdown", "Zahlen und BASIC", "Falten", "Art und Kodierung", "Formatieren"]),
    ("Ansicht", &["Bereiche", "Zoom und Schrift", "Farbschema", "Studio", "Tracker", "Reiter und Teilen"]),
    ("Projekt", &["Probleme und Protokoll", "Debuggen", "Aktive Datei"]),
];
/// Untermenü eines Befehls (leer: steht oben im Menü)
pub fn untermenue(c: Cmd) -> &'static str {
    use Cmd::*;
    match c {
        Suchen | Ersetzen | GeheZuZeile | InProjektSuchen | GeheZuDefinition => "Suchen und Ersetzen",
        MdFett | MdKursiv | MdCode | MdDurchgestrichen | MdLink | MdBild | MdUeberschriftTiefer | MdUeberschriftHoeher | MdAufzaehlung | MdNummerierung | MdAufgaben | MdZitat | MdAufgabeUmschalten | MdTabelleNeu | MdTabelleAusrichten | MdAlsMarkdown => "Markdown",
        Zahlenwerkzeuge | HexVorschlaege => "Zahlen und BASIC",
        AllesFalten | AllesEntfalten => "Falten",
        ArtText | ArtAsm | ArtBasic | Zeilenende | Kodierung => "Art und Kodierung",
        ZeileFormatieren | DateiFormatieren => "Formatieren",
        Seitenleiste | Schublade | Panel | MaximiereFokus | SpalteMax | DokumentMax | SchubladeMax | PanelMax | LayoutZuruecksetzen => "Bereiche",
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
pub fn menuename(i: &Info) -> &str { let u = untermenue(i.cmd); if u.is_empty() { i.name } else { i.name.strip_prefix(u).and_then(|r| r.strip_prefix(": ")).unwrap_or(i.name) } }
/// Befehle eines Menüs (`unter` leer: die obersten Einträge) in Tabellenreihenfolge
pub fn menue_eintraege<'a>(menue: &'a str, unter: &'a str) -> impl Iterator<Item = &'static Info> + 'a { BEFEHLE.iter().filter(move |i| i.menue == menue && untermenue(i.cmd) == unter) }
/// Untermenüs eines Menüs (leer bei Datei und Hilfe)
pub fn untermenues(menue: &str) -> &'static [&'static str] { UNTERMENUES.iter().find(|(m, _)| *m == menue).map_or(&[], |(_, u)| *u) }

/// Menü „Baum“ steht bewusst nicht hier: Befehle, die nur im Kontextmenü des Dateibaums sinnvoll sind, bleiben dort und in der Befehlspalette, stehen aber in keiner Menüleiste.
pub const MENUES: &[&str] = &["Datei", "Bearbeiten", "Ansicht", "Projekt", "Hilfe"];

pub fn info(c: Cmd) -> &'static Info { BEFEHLE.iter().find(|i| i.cmd == c).expect("Befehl fehlt in BEFEHLE") }
pub fn nach_id(id: &str) -> Option<Cmd> { BEFEHLE.iter().find(|i| i.id == id).map(|i| i.cmd) }

/// Kürzel zerlegt: (Cmd, Umschalt, Alt, Taste)
pub fn zerlege(tasten: &str) -> Option<(bool, bool, bool, &str)> {
    if tasten.is_empty() { return None }
    let mut p: Vec<&str> = tasten.split('+').collect(); let taste = p.pop()?;
    Some((p.contains(&"Cmd"), p.contains(&"Umschalt"), p.contains(&"Alt"), taste))
}

/// Anzeige je Plattform: macOS ⌘⇧⌥, sonst Strg+Umschalt+Alt
pub fn fuer_anzeige(tasten: &str, mac: bool) -> String {
    if mac { tasten.replace("Cmd+", "⌘").replace("Umschalt+", "⇧").replace("Alt+", "⌥").replace("Plus", "+").replace("Minus", "−").replace("Komma", ",").replace("Rechts", "→").replace("Links", "←") }
    else { tasten.replace("Cmd", "Strg").replace("Komma", ",") }
}

/// Freier Text mit Tastenangaben („Rückgängig (Cmd+Z)“, „Cmd/Strg+Z“): auf Windows und Linux wird „Cmd“ zu „Strg“, auf macOS bleibt der Text.
pub fn plattform_text(text: &str) -> String { plattform_text_fuer(text, cfg!(target_os = "macos")) }
pub fn plattform_text_fuer(text: &str, mac: bool) -> String { if mac || !text.contains("Cmd") { text.to_string() } else { text.replace("Cmd/Strg", "Strg").replace("Strg/Cmd", "Strg").replace("Cmd", "Strg") } }

/// Unscharfe Suche für die Palette: alle Wörter müssen vorkommen (Groß/Klein egal), Treffer am Wortanfang zuerst
pub fn filtere(frage: &str) -> Vec<&'static Info> {
    let q = frage.to_lowercase(); let w: Vec<&str> = q.split_whitespace().collect();
    let mut v: Vec<(u8, &Info)> = BEFEHLE.iter().filter_map(|i| { let n = i.name.to_lowercase(); if w.iter().all(|x| n.contains(x)) { Some((if w.first().map_or(true, |x| n.starts_with(x)) { 0 } else { 1 }, i)) } else { None } }).collect();
    v.sort_by_key(|x| x.0); v.into_iter().map(|x| x.1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test] fn menues_stimmen_und_jeder_befehl_hat_ein_menue() {
        for i in BEFEHLE { assert!(MENUES.contains(&i.menue) || i.menue == "Baum", "{} hat unbekanntes Menü {}", i.name, i.menue) }
        // Untermenüs: jeder Befehl eines Menüs steht genau einmal (oben oder in genau einem erklärten Untermenü), kein leeres Untermenü
        for i in BEFEHLE.iter().filter(|i| MENUES.contains(&i.menue)) { let u = untermenue(i.cmd); assert!(u.is_empty() || untermenues(i.menue).contains(&u), "{}: Untermenü „{}“ gehört nicht zu {}", i.name, u, i.menue) }
        for (m, subs) in UNTERMENUES { for u in *subs { assert!(menue_eintraege(m, u).count() > 0, "leeres Untermenü {m} ▸ {u}") } }
        for m in MENUES { let n = menue_eintraege(m, "").count() + untermenues(m).iter().map(|u| menue_eintraege(m, u).count()).sum::<usize>(); assert_eq!(n, BEFEHLE.iter().filter(|i| i.menue == *m).count(), "{m}: jeder Befehl genau einmal") }
        for m in ["Bearbeiten", "Ansicht", "Projekt"] { assert!(menue_eintraege(m, "").count() + untermenues(m).len() <= 18, "{m}: oberste Ebene zu lang") }
        // Menü „Datei“ bleibt übersichtlich; die Baum-Befehle stehen im Kontextmenü des Dateibaums
        assert!(BEFEHLE.iter().filter(|i| i.menue == "Datei").count() <= 18, "Menü Datei zu lang");
        for c in [Cmd::DateiUmbenennen, Cmd::DateiLoeschen, Cmd::PfadKopieren, Cmd::ImOrdnerZeigen, Cmd::HexAnsicht, Cmd::AlsInclude, Cmd::ZielOeffnen, Cmd::NeuerOrdner, Cmd::NeueNotizDialog] { assert_eq!(info(c).menue, "Baum", "{:?}", c); assert!(crate::kontext::eintraege(crate::kontext::Art::Baum).contains(&Some(c)) || crate::kontext::eintraege(crate::kontext::Art::BaumFlaeche).contains(&Some(c)), "{:?} bleibt im Kontextmenü des Baums", c) }
        for m in MENUES { assert!(BEFEHLE.iter().any(|i| i.menue == *m), "leeres Menü {m}") }
    }
    #[test] fn kuerzel_sind_eindeutig() {
        let mut seen: HashMap<&str, &str> = HashMap::new();
        for i in BEFEHLE.iter().filter(|i| !i.tasten.is_empty()) { if let Some(o) = seen.insert(i.tasten, i.name) { panic!("{} und {} teilen {}", o, i.name, i.tasten) } }
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
